# streamarg

Rust package representing either a standard stream (input/ output) or path argument.

## Usage

The [StreamPath] type represents either a path or standard stream.
It can be converted into a [StreamPathRead] or [StreamPathWrite],
which have some read-only or write-only convenience methods respectively,
or can open an [std::io::Read]er/ [std::io::BufRead]er,
or [std::io::Write]r/ buffered variant respectively.

Any of these types can be parsed from a string via [std:str::FromStr],
which is the stream if `"-"`, otherwise a [std::path::PathBuf].
They can also be converted directly from a [std::ffi::OsStr](ing),
in case your path isn't UTF-8.

### With [clap](https://crates.io/crates/clap)

Note that clap uses custom types' `FromStr` implementation,
so paths must be UTF-8.

```rust
use clap::Parser;
use streampath::{StreamPathRead, StreamPathWrite};
use std::io::Write;

#[derive(Debug, Parser)]
struct Args {
    in_file: StreamPathRead,
    out_file: Option<StreamPathWrite>,
}

let args = Args::parse_from(["mycli", "README.md"]);

let in_content = args.in_file.read_to_string().unwrap();

// defaults to Stream variant
let mut out_file = args.out_file.unwrap_or_default();
let mut out_writer = out_file.writer().unwrap();
write!(&mut out_writer, "I read {in_content}").unwrap();
```

## cargo features

### `buf`

Adds support for wrapping a `Vec<u8>`, useful for testing purposes.
Inspired by `take_bytes`.

## Alternatives

- [clap-stdin](https://crates.io/crates/clap-stdin)
  - Uses dynamic dispatch for the readers/writers rather than an enum
  - Stores the path internally as a String (i.e. must be UTF-8)
  - Provides more guarantees around multiple users of the same stream
- [take_bytes](https://crates.io/crates/take_bytes)
  - only reads
