use std::fmt::Display;
use std::{
    convert::Infallible,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{StreamPathRead, StreamPathWrite};

/// Representation of either a path or a stream (stdin/stdout).
/// Can be used to create (Buf)Readers and (Buf)Writers.
///
/// There are no guarantees about multiple uses of the same stream.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StreamPath {
    Path(PathBuf),
    #[default]
    Stream,
    /// In-memory buffer. Reading reads from the start of the buffer;
    /// writing truncates the buffer then appends to it.
    #[cfg(feature = "buf")]
    Buf(Vec<u8>),
}

impl Display for StreamPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamPath::Path(p) => write!(f, "{}", p.display()),
            StreamPath::Stream => write!(f, "-"),
            #[cfg(feature = "buf")]
            StreamPath::Buf(_) => write!(f, "<buffer>"),
        }
    }
}

impl StreamPath {
    /// Like [FromStr], but for OS strings (which may not be valid UTF-8).
    ///
    /// `-` becomes [StreamPath::Stream]; anything else is treated as a path.
    pub fn from_os_str(s: impl AsRef<OsStr>) -> Self {
        s.as_ref().into()
    }

    /// The path of the file, or `None` if this represents a stream (or buffer).
    pub fn path(&self) -> Option<&Path> {
        match self {
            StreamPath::Path(path) => Some(path.as_path()),
            _ => None,
        }
    }

    /// The in-memory buffer, if this represents one.
    #[cfg(feature = "buf")]
    pub fn buf(&self) -> Option<&[u8]> {
        match self {
            StreamPath::Buf(b) => Some(b),
            _ => None,
        }
    }

    /// Convert into a [StreamPathRead].
    pub fn read_only(self) -> StreamPathRead {
        StreamPathRead::from(self)
    }

    /// Convert into a [StreamPathWrite].
    pub fn write_only(self) -> StreamPathWrite {
        StreamPathWrite::from(self)
    }
}

#[cfg(feature = "buf")]
impl StreamPath {
    /// Create from an in-memory buffer.
    pub fn new_buf(buf: impl Into<Vec<u8>>) -> Self {
        StreamPath::Buf(buf.into())
    }

    /// Create from an empty in-memory buffer.
    pub fn new_empty_buf() -> Self {
        StreamPath::Buf(Vec::new())
    }
}

#[cfg(feature = "buf")]
impl From<Vec<u8>> for StreamPath {
    fn from(buf: Vec<u8>) -> Self {
        StreamPath::Buf(buf)
    }
}

impl From<&Path> for StreamPath {
    fn from(path: &Path) -> Self {
        StreamPath::Path(path.to_path_buf())
    }
}

impl From<PathBuf> for StreamPath {
    fn from(path: PathBuf) -> Self {
        StreamPath::Path(path)
    }
}

/// `-` becomes [StreamPath::Stream]; anything else is treated as a path.
impl From<&OsStr> for StreamPath {
    fn from(s: &OsStr) -> Self {
        if s == "-" {
            StreamPath::Stream
        } else {
            StreamPath::Path(PathBuf::from(s))
        }
    }
}

/// `-` becomes [StreamPath::Stream]; anything else is treated as a path.
/// Does not reallocate.
impl From<OsString> for StreamPath {
    fn from(s: OsString) -> Self {
        if s == "-" {
            StreamPath::Stream
        } else {
            StreamPath::Path(PathBuf::from(s))
        }
    }
}

impl FromStr for StreamPath {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_os_str(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_dash_is_stream() {
        let sp: StreamPath = "-".parse().unwrap();
        assert_eq!(sp, StreamPath::Stream);
        assert_eq!(sp.path(), None);
    }

    #[test]
    fn from_str_path() {
        let sp: StreamPath = "some/file.txt".parse().unwrap();
        assert_eq!(sp, StreamPath::Path(PathBuf::from("some/file.txt")));
        assert_eq!(sp.path(), Some(Path::new("some/file.txt")));
    }

    #[test]
    fn from_str_only_exact_dash_is_stream() {
        for s in ["--", "./-", " -", "-.txt"] {
            let sp: StreamPath = s.parse().unwrap();
            assert_eq!(sp, StreamPath::Path(PathBuf::from(s)), "input {s:?}");
        }
    }

    #[test]
    fn from_os_str_matches_from_str() {
        for s in ["-", "--", "./-", "some/file.txt"] {
            let expected: StreamPath = s.parse().unwrap();
            assert_eq!(StreamPath::from_os_str(s), expected);
            assert_eq!(StreamPath::from_os_str(OsStr::new(s)), expected);
            assert_eq!(
                StreamPath::from_os_str(std::ffi::OsString::from(s)),
                expected
            );
            assert_eq!(StreamPath::from(OsStr::new(s)), expected);
            assert_eq!(StreamPath::from(OsString::from(s)), expected);
            assert_eq!(StreamPathRead::from(OsStr::new(s)).into_inner(), expected);
            assert_eq!(
                StreamPathWrite::from(OsString::from(s)).into_inner(),
                expected
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn from_os_str_non_utf8() {
        use std::os::unix::ffi::OsStrExt;
        let raw = OsStr::from_bytes(b"bad\xffname");
        assert_eq!(
            StreamPath::from_os_str(raw),
            StreamPath::Path(PathBuf::from(raw))
        );
    }

    #[test]
    fn display_round_trips_through_from_str() {
        for s in ["-", "a/b/c.txt"] {
            let sp: StreamPath = s.parse().unwrap();
            assert_eq!(sp.to_string(), s);
        }
    }

    #[test]
    fn read_only_and_write_only_preserve_inner() {
        let sp = StreamPath::from(PathBuf::from("x.txt"));
        assert_eq!(sp.clone().read_only().into_inner(), sp);
        assert_eq!(sp.clone().write_only().into_inner(), sp);

        assert_eq!(
            StreamPath::Stream.read_only().into_inner(),
            StreamPath::Stream
        );
        assert_eq!(
            StreamPath::Stream.write_only().into_inner(),
            StreamPath::Stream
        );
    }

    #[cfg(feature = "buf")]
    #[test]
    fn buf_display_and_accessors() {
        let sp = StreamPath::new_buf(b"abc".to_vec());
        assert_eq!(sp.to_string(), "<buffer>");
        assert_eq!(sp.path(), None);
        assert_eq!(sp.buf(), Some(&b"abc"[..]));

        assert_eq!(StreamPath::Stream.buf(), None);
        assert_eq!(StreamPath::from(PathBuf::from("a")).buf(), None);
    }

    #[cfg(feature = "buf")]
    #[test]
    fn buf_constructors_agree() {
        let v = b"data".to_vec();
        let expected = StreamPath::Buf(v.clone());
        assert_eq!(StreamPath::new_buf(v.clone()), expected);
        assert_eq!(StreamPath::from(v), expected);
        assert_eq!(StreamPath::new_empty_buf(), StreamPath::Buf(Vec::new()));
    }
}
