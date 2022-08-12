//! Simple MLLP implementation
//!
//! "The goal of the MLLP Message Transport protocol is to provide an interface between HL7
//! Applications and the transport protocol that uses minimal overhead." (from *HL7 Version 3 Standard:
//! Transport Specification - MLLP, Release 2*).
//!
//! MLLP is a simple protocol used for transmitting HL7 messages between HL7 applications. It goes
//! like this `<SB>...<EB><CR>`, where:
//! - SB is the Start Block Character, 0x0B.
//! - EB is the End Block Character, 0x1C.
//! - CR is the Carriage Return Character, 0x0D.
//! This is called the Block Format.
//!
//! MLLP contains 2 other formats, the Commit Acknowledgement
//! Block `<SB><ACK><EB><CR>`, and the Negative Commit Acknowledgement Block `<SB><NACK><EB><CR>`,
//! where:
//! - ACK is the acknowledgement character, 0x06.
//! - NAK is the negative-acknowledgement character, 0x15.
//!
//! # Quick start
//!
//! Client side code might look like this:
//! ```
//! use std::io::prelude::*;
//! use std::net::TcpStream;
//! use mllp_rs::MllpCodec;
//!
//! // Client side
//! let mut stream = TcpStream::connect("127.0.0.1:5000")?;
//! let _ = stream.write(MllpCodec::encode("MSH|^~\&|WIR|||36|20200514123930||VXU^V04^VXU_V04|43|P|2.5.1|||ER").as_bytes());
//! ```
//!
//! Server side code might look like this:
//! ```
//! use std::io::prelude::*;
//! use std::net::TcpListener;
//! use mllp_rs::MllpCodec;
//!
//! let mut listener = TcpListener::bind(addr).unwrap();
//! for stream in listener.incoming() {
//!     let mut buf: Vec<u8> = vec![];
//!     let _ = stream?.read_to_end(&mut buf);
//!     let decoded_data = String::from_utf8_lossy(MllpCodec::decode(buf.as_slice())?);
//! }
//! ```


use std::fmt;
const SB: char = 11u8 as char;
const EB: char = 28u8 as char;
const CR: char = 13u8 as char;
const ACK: char = 6u8 as char;
const NAK: char = 15u8 as char;

pub struct MllpCodec { }

impl MllpCodec {
    pub fn encode(with: &str) -> String {
        format!("{}{}{}{}", SB, with, EB, CR)
    }

    pub fn decode(with: &[u8]) -> Result<&[u8], MllpSyntaxError> {
        assert!(with.len() >= 4);

        let sb = with[0] as char;
        let hl7 = &with[1..with.len() - 2];
        let eb = with[with.len() - 2] as char;
        let cr = with[with.len() - 1] as char;

        if sb == SB && eb == EB && cr == CR {
            Ok(hl7)
        } else {
            Err(MllpSyntaxError)
        }
    }

    /// Creates an MLLP ACK.
    /// ```
    /// use mllp_rs::MllpCodec;
    ///
    /// let ack = MllpCodec::ack();
    /// ```
    pub fn ack() -> String {
        format!("{}{}{}{}", SB, ACK, EB, CR)
    }

    /// Creates an MLLP NAK (Negative ACK).
    /// ```
    /// use mllp_rs::MllpCodec;
    ///
    /// let nak = MllpCodec::nak();
    /// ```
    pub fn nak() -> String {
        format!("{}{}{}{}", SB, NAK, EB, CR)
    }

    pub fn is_ack(with: &[u8]) -> bool {
        with == Self::ack().as_bytes()
    }

    pub fn is_nak(with: &[u8]) -> bool {
        with == Self::nak().as_bytes()
    }
}

#[derive(Debug)]
pub struct MllpSyntaxError;

impl fmt::Display for MllpSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expected bytes <SB>...<EB><CR>")
    }
}

impl std::error::Error for MllpSyntaxError { }

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use crate::MllpCodec;

    #[test]
    fn encode_and_decode_same_message() {
        let data = "MSH|^~\\&|ZIS|1^AHospital|||200405141144||¶ADT^A01|20041104082400|P|2.3|||AL|NE|||8859/15|¶EVN|A01|20041104082400.0000+0100|20041104082400¶PID||\"\"|10||Vries^Danny^D.^^de||19951202|M|||Rembrandlaan^7^Leiden^^7301TH^\"\"^^P||\"\"|\"\"||\"\"|||||||\"\"|\"\"¶PV1||I|3w^301^\"\"^01|S|||100^van den Berg^^A.S.^^\"\"^dr|\"\"||9||||H||||20041104082400.0000+0100";
        let encoded_data = MllpCodec::encode(data);
        let decoded_data = MllpCodec::decode(encoded_data.as_bytes());

        assert!(decoded_data.is_ok());
        assert_eq!(decoded_data.unwrap(), data.as_bytes());
    }

    #[test]
    fn listen_and_receive_mllp_packet() {
        let data = "MSH|^~\\&|ZIS|1^AHospital|||200405141144||¶ADT^A01|20041104082400|P|2.3|||AL|NE|||8859/15|¶EVN|A01|20041104082400.0000+0100|20041104082400¶PID||\"\"|10||Vries^Danny^D.^^de||19951202|M|||Rembrandlaan^7^Leiden^^7301TH^\"\"^^P||\"\"|\"\"||\"\"|||||||\"\"|\"\"¶PV1||I|3w^301^\"\"^01|S|||100^van den Berg^^A.S.^^\"\"^dr|\"\"||9||||H||||20041104082400.0000+0100";
        let original_data = data.clone();
        let addr = "127.0.0.1:5000";
        let (tx, rx) = mpsc::channel();

        let handler = thread::spawn(move || {
            let listener = TcpListener::bind(addr).unwrap();
            tx.send(true).unwrap();

            for stream in listener.incoming() {
                assert!(stream.is_ok());
                let mut buf: Vec<u8> = vec![];
                let _ = stream.unwrap().read_to_end(&mut buf);
                let decoded_data = String::from_utf8_lossy(MllpCodec::decode(buf.as_slice()).unwrap());
                assert_eq!(decoded_data, data);
                break;
            }
            // close the socket server
            drop(listener);
        });

        let handler2 = thread::spawn(move || {
            for received in rx {
                if received {
                    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 5000));
                    let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(3)).unwrap();
                    let _ = stream.write(MllpCodec::encode(original_data).as_bytes());
                }
            }

        });

        handler2.join().expect("TODO: panic message server");
        handler.join().expect("TODO: panic message listener");
    }

    #[test]
    fn it_creates_ack() {
        let ack = MllpCodec::ack();
        assert!(MllpCodec::is_ack(ack.as_bytes()));
    }

    #[test]
    fn it_creates_nak() {
        let nak = MllpCodec::nak();
        assert!(MllpCodec::is_nak(nak.as_bytes()));
    }
}
