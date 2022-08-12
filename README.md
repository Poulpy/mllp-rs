# mllp-rs

mllp-rs is an MLLP implementation in Rust, for the HL7 file standard. It encapsulates
the HL7 file in network transmissions. The library provide utilities for encoding and
decoding an encapsulated HL7 file. It also provides helpers to generate MLLP ACK and
MLLP NAK.

## Install

Put this in your Cargo.toml under `[dependecies]`:
```toml
mllp-rs = "0.1.0"
```

## Get started

To encode an HL7 file use `MllpCodec::encode()`, to decode use `MllpCodec::decode()`.

Client side code might look like this:
```rust
use std::io::prelude::*;
use std::net::TcpStream;
use mllp_rs::MllpCodec;

// Client side
let mut stream = TcpStream::connect("127.0.0.1:5000")?;
let _ = stream.write(MllpCodec::encode("MSH|^~\&|WIR|||36|20200514123930||VXU^V04^VXU_V04|43|P|2.5.1|||ER").as_bytes());
```

Server side code might look like this:
```rust
use std::io::prelude::*;
use std::net::TcpListener;
use mllp_rs::MllpCodec;

let mut listener = TcpListener::bind(addr).unwrap();
for stream in listener.incoming() {
    let mut buf: Vec<u8> = vec![];
    let _ = stream?.read_to_end(&mut buf);
    let decoded_data = String::from_utf8_lossy(MllpCodec::decode(buf.as_slice())?);
}
```

## Misc

You might want to check out also [hl7-mllp-codec](https://github.com/wokket/hl7-mllp-codec) !
