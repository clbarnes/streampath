# streamarg

Rust package representing either a standard stream (input/ output) or path argument.

## Usage

### With clap

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
