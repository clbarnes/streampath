#[cfg(feature = "buf")]
use std::io::Cursor;
use std::{
    fs::File,
    io::{BufRead, Read, Stdin, StdinLock},
    path::Path,
};

use crate::StreamPath;

/// A [StreamPath] which can only be read from (stream is STDIN).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamPathRead(StreamPath);

impl StreamPathRead {
    /// Read the entire contents of the file or stream.
    pub fn read_all(&self) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::default();
        let mut reader = self.reader()?;
        reader.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Read the entire UTF-8 contents of the file or stream.
    pub fn read_to_string(&self) -> std::io::Result<String> {
        let mut out = String::default();
        let mut reader = self.reader()?;
        reader.read_to_string(&mut out)?;
        Ok(out)
    }

    /// Get a reader for the file, or STDIN.
    pub fn reader(&self) -> std::io::Result<impl Read> {
        #[cfg(feature = "buf")]
        if let Some(b) = self.buf() {
            return Ok(Reader::new_buf(b));
        }
        match self.path() {
            Some(p) => Reader::new_file(p),
            None => Ok(Reader::new_stdin()),
        }
    }

    /// Get a buffered reader of the file, or locked STDIN.
    pub fn buf_reader(&self) -> std::io::Result<impl BufRead> {
        #[cfg(feature = "buf")]
        if let Some(b) = self.buf() {
            return Ok(BufReader::new_buf(b));
        }
        match self.path() {
            Some(p) => BufReader::new_file(p),
            None => Ok(BufReader::new_stdin()),
        }
    }
}

impl_newtype_conversions!(StreamPathRead);

pub(crate) enum Reader<'a> {
    Stdin(Stdin),
    File(File),
    #[cfg(feature = "buf")]
    Buf(Cursor<&'a [u8]>),
    #[cfg(not(feature = "buf"))]
    #[allow(dead_code)]
    _Phantom(std::marker::PhantomData<&'a ()>),
}

impl<'a> Reader<'a> {
    pub(crate) fn new_stdin() -> Self {
        Reader::Stdin(std::io::stdin())
    }

    pub(crate) fn new_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(Reader::File(file))
    }

    #[cfg(feature = "buf")]
    pub(crate) fn new_buf(buf: &'a [u8]) -> Self {
        Reader::Buf(Cursor::new(buf))
    }
}

impl Read for Reader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Reader::Stdin(stdin) => stdin.read(buf),
            Reader::File(file) => file.read(buf),
            #[cfg(feature = "buf")]
            Reader::Buf(cursor) => cursor.read(buf),
            #[cfg(not(feature = "buf"))]
            Reader::_Phantom(_) => unreachable!(),
        }
    }
}

pub(crate) enum BufReader<'a> {
    Stdin(StdinLock<'static>),
    File(std::io::BufReader<File>),
    #[cfg(feature = "buf")]
    Buf(Cursor<&'a [u8]>),
    #[cfg(not(feature = "buf"))]
    #[allow(dead_code)]
    _Phantom(std::marker::PhantomData<&'a ()>),
}

impl<'a> BufReader<'a> {
    pub(crate) fn new_stdin() -> Self {
        BufReader::Stdin(std::io::stdin().lock())
    }

    pub(crate) fn new_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(BufReader::File(std::io::BufReader::new(file)))
    }

    #[cfg(feature = "buf")]
    pub(crate) fn new_buf(buf: &'a [u8]) -> Self {
        BufReader::Buf(Cursor::new(buf))
    }
}

impl Read for BufReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            BufReader::Stdin(stdin) => stdin.read(buf),
            BufReader::File(file) => file.read(buf),
            #[cfg(feature = "buf")]
            BufReader::Buf(cursor) => cursor.read(buf),
            #[cfg(not(feature = "buf"))]
            BufReader::_Phantom(_) => unreachable!(),
        }
    }
}

impl BufRead for BufReader<'_> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            BufReader::Stdin(stdin) => stdin.fill_buf(),
            BufReader::File(file) => file.fill_buf(),
            #[cfg(feature = "buf")]
            BufReader::Buf(cursor) => cursor.fill_buf(),
            #[cfg(not(feature = "buf"))]
            BufReader::_Phantom(_) => unreachable!(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            BufReader::Stdin(stdin) => stdin.consume(amt),
            BufReader::File(file) => file.consume(amt),
            #[cfg(feature = "buf")]
            BufReader::Buf(cursor) => cursor.consume(amt),
            #[cfg(not(feature = "buf"))]
            BufReader::_Phantom(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: &str = "first line\nsecond line\nthird line\n";

    #[test]
    fn from_str() {
        let r: StreamPathRead = "-".parse().unwrap();
        assert_eq!(r.into_inner(), StreamPath::Stream);

        let r: StreamPathRead = "in.txt".parse().unwrap();
        assert_eq!(r.path(), Some(Path::new("in.txt")));
    }

    #[test]
    fn display_round_trips_through_from_str() {
        for s in ["-", "a/b/c.txt"] {
            let r: StreamPathRead = s.parse().unwrap();
            assert_eq!(r.to_string(), s);
        }
    }

    #[test]
    fn from_stream_path_and_back() {
        let sp = StreamPath::from(Path::new("y.txt"));
        let r = StreamPathRead::from(sp.clone());
        assert_eq!(StreamPath::from(r), sp);
    }

    #[test]
    fn from_paths() {
        let p = Path::new("z.txt");
        assert_eq!(StreamPathRead::from(p).path(), Some(p));
        assert_eq!(StreamPathRead::from(p.to_path_buf()).path(), Some(p));
    }

    #[test]
    fn default_is_stream() {
        assert_eq!(StreamPathRead::default().into_inner(), StreamPath::Stream);
    }

    #[test]
    fn read_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("in.txt");
        std::fs::write(&path, CONTENT).unwrap();

        let r = StreamPathRead::from(path);
        assert_eq!(r.read_to_string().unwrap(), CONTENT);
        assert_eq!(r.read_all().unwrap(), CONTENT.as_bytes());

        let mut s = String::new();
        r.reader().unwrap().read_to_string(&mut s).unwrap();
        assert_eq!(s, CONTENT);

        let lines: Vec<String> = r
            .buf_reader()
            .unwrap()
            .lines()
            .map(Result::unwrap)
            .collect();
        assert_eq!(lines, ["first line", "second line", "third line"]);
    }

    #[test]
    fn reading_missing_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        let r = StreamPathRead::from(dir.path().join("does_not_exist.txt"));

        assert_eq!(
            r.read_all().unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        assert!(r.reader().is_err());
        assert!(r.buf_reader().is_err());
    }

    #[cfg(feature = "buf")]
    mod buf {
        use super::*;

        #[test]
        fn constructors_agree() {
            let v = b"data".to_vec();
            let expected = StreamPath::Buf(v.clone());
            assert_eq!(StreamPathRead::new_buf(v.clone()).into_inner(), expected);
            assert_eq!(StreamPathRead::from(v).into_inner(), expected);
            assert_eq!(StreamPathRead::new_empty_buf().buf(), Some(&[][..]));
        }

        #[test]
        fn read_prepopulated_buf() {
            let r = StreamPathRead::new_buf(CONTENT.as_bytes().to_vec());

            assert_eq!(r.read_all().unwrap(), CONTENT.as_bytes());
            assert_eq!(r.read_to_string().unwrap(), CONTENT);

            let mut s = String::new();
            r.reader().unwrap().read_to_string(&mut s).unwrap();
            assert_eq!(s, CONTENT);

            let lines: Vec<String> = r
                .buf_reader()
                .unwrap()
                .lines()
                .map(Result::unwrap)
                .collect();
            assert_eq!(lines, ["first line", "second line", "third line"]);
        }

        #[test]
        fn reading_is_repeatable_and_non_destructive() {
            let r = StreamPathRead::new_buf(CONTENT.as_bytes().to_vec());
            assert_eq!(r.read_to_string().unwrap(), CONTENT);
            assert_eq!(r.read_to_string().unwrap(), CONTENT);
            assert_eq!(r.buf(), Some(CONTENT.as_bytes()));
        }

        #[test]
        fn read_empty_buf() {
            let r = StreamPathRead::new_empty_buf();
            assert_eq!(r.read_all().unwrap(), b"");
            assert_eq!(r.buf_reader().unwrap().lines().count(), 0);
        }
    }
}
