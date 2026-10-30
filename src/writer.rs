use std::{
    fs::File,
    io::{Stdout, StdoutLock, Write},
    path::Path,
};

use crate::StreamPath;

/// A [StreamPath] which can only be written to (stream is STDOUT).
///
/// Methods take `&mut self` so that in-memory buffers can be written to.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamPathWrite(StreamPath);

impl StreamPathWrite {
    /// Write the entire contents of the buffer to the file or stream.
    pub fn write(&mut self, buf: impl AsRef<[u8]>) -> std::io::Result<()> {
        let mut writer = self.writer()?;
        writer.write_all(buf.as_ref())?;
        writer.flush()
    }

    /// Get a writer for the file, or STDOUT.
    pub fn writer(&mut self) -> std::io::Result<impl Write> {
        match &mut self.0 {
            StreamPath::Path(p) => Writer::new_file(p),
            StreamPath::Stream => Ok(Writer::new_stdout()),
            #[cfg(feature = "buf")]
            StreamPath::Buf(b) => Ok(Writer::new_buf(b)),
        }
    }

    /// Get a buffered writer of the file, or locked STDOUT.
    pub fn buf_writer(&mut self) -> std::io::Result<impl Write> {
        match &mut self.0 {
            StreamPath::Path(p) => BufWriter::new_file(p),
            StreamPath::Stream => Ok(BufWriter::new_stdout()),
            #[cfg(feature = "buf")]
            StreamPath::Buf(b) => Ok(BufWriter::new_buf(b)),
        }
    }
}

impl_newtype_conversions!(StreamPathWrite);

pub(crate) enum Writer<'a> {
    Stdout(Stdout),
    File(File),
    #[cfg(feature = "buf")]
    Buf(&'a mut Vec<u8>),
    #[cfg(not(feature = "buf"))]
    #[allow(dead_code)]
    _Phantom(std::marker::PhantomData<&'a ()>),
}

impl<'a> Writer<'a> {
    pub(crate) fn new_stdout() -> Self {
        Writer::Stdout(std::io::stdout())
    }

    pub(crate) fn new_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Writer::File(file))
    }

    /// Truncates the buffer (like [File::create]) and appends to it.
    #[cfg(feature = "buf")]
    pub(crate) fn new_buf(buf: &'a mut Vec<u8>) -> Self {
        buf.clear();
        Writer::Buf(buf)
    }
}

impl Write for Writer<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Writer::Stdout(stdout) => stdout.write(buf),
            Writer::File(file) => file.write(buf),
            #[cfg(feature = "buf")]
            Writer::Buf(v) => v.write(buf),
            #[cfg(not(feature = "buf"))]
            Writer::_Phantom(_) => unreachable!(),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Writer::Stdout(stdout) => stdout.flush(),
            Writer::File(file) => file.flush(),
            #[cfg(feature = "buf")]
            Writer::Buf(v) => v.flush(),
            #[cfg(not(feature = "buf"))]
            Writer::_Phantom(_) => unreachable!(),
        }
    }
}

pub(crate) enum BufWriter<'a> {
    Stdout(StdoutLock<'static>),
    File(std::io::BufWriter<File>),
    #[cfg(feature = "buf")]
    Buf(&'a mut Vec<u8>),
    #[cfg(not(feature = "buf"))]
    #[allow(dead_code)]
    _Phantom(std::marker::PhantomData<&'a ()>),
}

impl<'a> BufWriter<'a> {
    pub(crate) fn new_stdout() -> Self {
        BufWriter::Stdout(std::io::stdout().lock())
    }

    pub(crate) fn new_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(BufWriter::File(std::io::BufWriter::new(file)))
    }

    /// Truncates the buffer (like [File::create]) and appends to it.
    /// No extra buffering is needed for an in-memory Vec.
    #[cfg(feature = "buf")]
    pub(crate) fn new_buf(buf: &'a mut Vec<u8>) -> Self {
        buf.clear();
        BufWriter::Buf(buf)
    }
}

impl Write for BufWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            BufWriter::Stdout(stdout) => stdout.write(buf),
            BufWriter::File(file) => file.write(buf),
            #[cfg(feature = "buf")]
            BufWriter::Buf(v) => v.write(buf),
            #[cfg(not(feature = "buf"))]
            BufWriter::_Phantom(_) => unreachable!(),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            BufWriter::Stdout(stdout) => stdout.flush(),
            BufWriter::File(file) => file.flush(),
            #[cfg(feature = "buf")]
            BufWriter::Buf(v) => v.flush(),
            #[cfg(not(feature = "buf"))]
            BufWriter::_Phantom(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StreamPathRead;

    const CONTENT: &str = "first line\nsecond line\nthird line\n";

    #[test]
    fn from_str() {
        let w: StreamPathWrite = "-".parse().unwrap();
        assert_eq!(w.into_inner(), StreamPath::Stream);

        let w: StreamPathWrite = "out.txt".parse().unwrap();
        assert_eq!(w.path(), Some(Path::new("out.txt")));
    }

    #[test]
    fn display_round_trips_through_from_str() {
        for s in ["-", "a/b/c.txt"] {
            let w: StreamPathWrite = s.parse().unwrap();
            assert_eq!(w.to_string(), s);
        }
    }

    #[test]
    fn from_stream_path_and_back() {
        let sp = StreamPath::from(Path::new("y.txt"));
        let w = StreamPathWrite::from(sp.clone());
        assert_eq!(StreamPath::from(w), sp);
    }

    #[test]
    fn from_paths() {
        let p = Path::new("z.txt");
        assert_eq!(StreamPathWrite::from(p).path(), Some(p));
        assert_eq!(StreamPathWrite::from(p.to_path_buf()).path(), Some(p));
    }

    #[test]
    fn default_is_stream() {
        assert_eq!(StreamPathWrite::default().into_inner(), StreamPath::Stream);
    }

    #[test]
    fn write_then_read_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.txt");

        let mut w = StreamPathWrite::from(path.clone());
        w.write(CONTENT).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), CONTENT);

        let r = StreamPathRead::from(path);
        assert_eq!(r.read_to_string().unwrap(), CONTENT);
    }

    #[test]
    fn file_writer_and_buf_writer() {
        let dir = tempfile::tempdir().unwrap();

        let path = dir.path().join("writer.txt");
        let mut w = StreamPathWrite::from(path.clone());
        {
            let mut writer = w.writer().unwrap();
            writer.write_all(CONTENT.as_bytes()).unwrap();
            writer.flush().unwrap();
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), CONTENT);

        let path = dir.path().join("buf_writer.txt");
        let mut w = StreamPathWrite::from(path.clone());
        {
            let mut writer = w.buf_writer().unwrap();
            for line in CONTENT.lines() {
                writeln!(writer, "{line}").unwrap();
            }
            writer.flush().unwrap();
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), CONTENT);
    }

    #[test]
    fn writing_truncates_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("truncate.txt");
        std::fs::write(&path, "this is much longer than the replacement").unwrap();

        StreamPathWrite::from(path.clone()).write("short").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "short");
    }

    #[test]
    fn writing_to_missing_directory_errors() {
        let dir = tempfile::tempdir().unwrap();
        let mut w = StreamPathWrite::from(dir.path().join("no_such_dir").join("out.txt"));
        assert!(w.write("x").is_err());
    }

    #[cfg(feature = "buf")]
    mod buf {
        use super::*;

        #[test]
        fn constructors_agree() {
            let v = b"data".to_vec();
            let expected = StreamPath::Buf(v.clone());
            assert_eq!(StreamPathWrite::new_buf(v.clone()).into_inner(), expected);
            assert_eq!(StreamPathWrite::from(v).into_inner(), expected);
            assert_eq!(StreamPathWrite::new_empty_buf().buf(), Some(&[][..]));
        }

        #[test]
        fn write_to_empty_buf_then_validate() {
            let mut w = StreamPathWrite::new_empty_buf();
            w.write(CONTENT).unwrap();

            assert_eq!(w.buf(), Some(CONTENT.as_bytes()));
            assert_eq!(w.into_inner(), StreamPath::Buf(CONTENT.as_bytes().to_vec()));
        }

        #[test]
        fn writer_and_buf_writer_to_buf() {
            let mut w = StreamPathWrite::new_empty_buf();
            {
                let mut writer = w.writer().unwrap();
                write!(writer, "hello ").unwrap();
                write!(writer, "world").unwrap();
                writer.flush().unwrap();
            }
            assert_eq!(w.buf(), Some(&b"hello world"[..]));

            let mut w = StreamPathWrite::new_empty_buf();
            {
                let mut writer = w.buf_writer().unwrap();
                for line in CONTENT.lines() {
                    writeln!(writer, "{line}").unwrap();
                }
                writer.flush().unwrap();
            }
            assert_eq!(w.buf(), Some(CONTENT.as_bytes()));
        }

        #[test]
        fn writing_truncates_prepopulated_buf() {
            let mut w = StreamPathWrite::new_buf(b"old content which is long".to_vec());
            w.write("new").unwrap();
            assert_eq!(w.buf(), Some(&b"new"[..]));
        }

        #[test]
        fn each_writer_truncates_buf() {
            let mut w = StreamPathWrite::new_empty_buf();
            w.write("first").unwrap();
            w.write("second").unwrap();
            assert_eq!(w.buf(), Some(&b"second"[..]));
        }

        #[test]
        fn written_buf_can_be_read_back() {
            let mut w = StreamPathWrite::new_empty_buf();
            w.write(CONTENT).unwrap();

            let r = w.into_inner().read_only();
            assert_eq!(r.read_to_string().unwrap(), CONTENT);
        }
    }
}
