/// Implements constructors, conversions and [Display](std::fmt::Display)
/// for a newtype over [StreamPath](crate::StreamPath).
macro_rules! impl_newtype_conversions {
    ($t:ident) => {
        impl $t {
            /// The path of the file, or `None` if this represents a stream (or buffer).
            pub fn path(&self) -> Option<&std::path::Path> {
                self.0.path()
            }

            /// The in-memory buffer, if this represents one.
            #[cfg(feature = "buf")]
            pub fn buf(&self) -> Option<&[u8]> {
                self.0.buf()
            }

            pub fn into_inner(self) -> $crate::StreamPath {
                self.0
            }
        }

        #[cfg(feature = "buf")]
        impl $t {
            /// Create from an in-memory buffer.
            pub fn new_buf(buf: impl Into<Vec<u8>>) -> Self {
                $t($crate::StreamPath::new_buf(buf))
            }

            /// Create from an empty in-memory buffer.
            pub fn new_empty_buf() -> Self {
                $t($crate::StreamPath::new_empty_buf())
            }
        }

        #[cfg(feature = "buf")]
        impl From<Vec<u8>> for $t {
            fn from(buf: Vec<u8>) -> Self {
                $t::new_buf(buf)
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<$crate::StreamPath> for $t {
            fn from(sp: $crate::StreamPath) -> Self {
                $t(sp)
            }
        }

        impl From<$t> for $crate::StreamPath {
            fn from(t: $t) -> Self {
                t.0
            }
        }

        impl From<&std::path::Path> for $t {
            fn from(path: &std::path::Path) -> Self {
                $t(path.into())
            }
        }

        impl From<std::path::PathBuf> for $t {
            fn from(path: std::path::PathBuf) -> Self {
                $t(path.into())
            }
        }

        impl std::str::FromStr for $t {
            type Err = std::convert::Infallible;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse().map($t)
            }
        }
    };
}
