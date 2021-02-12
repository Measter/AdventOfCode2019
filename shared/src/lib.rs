#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![warn(clippy::pedantic)]

#[cfg(any(feature = "std", test))]
extern crate std;

#[cfg(any(feature = "std", test))]
mod compress;
mod decompress;
mod input;

pub use input::*;

const ADDR_SIZE: usize = 2;
const DICT_START_ADDR: core::ops::Range<usize> = 0..2;
const RECORD_START_ADDR: core::ops::Range<usize> = 2..4;
const NUM_RECORD_ADDR: core::ops::Range<usize> = 4..6;
const LOOKUP_START: usize = 6;

const RUN_LEN_MAX_BYTE: u16 = 0x7F;
const MULTI_BYTE_START: u8 = 0x80;

pub enum ErrorKind {
    InvalidCompressedFlag,
    LengthDecode,
    RecordReadError,
    #[cfg(any(feature = "std", test))]
    Io(std::io::Error),
}

#[cfg(any(feature = "std", test))]
impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Debug, Copy, Clone)]
enum RunLengthEncoded {
    Single([u8; 1]),
    Double([u8; 2]),
}

impl AsRef<[u8]> for RunLengthEncoded {
    fn as_ref(&self) -> &[u8] {
        match self {
            RunLengthEncoded::Single(v) => v.as_ref(),
            RunLengthEncoded::Double(v) => v.as_ref(),
        }
    }
}

impl RunLengthEncoded {
    #[must_use]
    fn encode(val: u16) -> RunLengthEncoded {
        // We'll be needing the top bit for the multi-byte marker.
        assert!(val < 0x7FFF);

        if val > RUN_LEN_MAX_BYTE {
            let [lb, hb] = val.to_le_bytes();
            RunLengthEncoded::Double([hb | MULTI_BYTE_START, lb])
        } else {
            RunLengthEncoded::Single([val.to_le_bytes()[0]])
        }
    }

    #[must_use]
    fn decode(bytes: &[u8]) -> Option<(u16, &[u8])> {
        match bytes {
            [val @ 0..=0x7F, xs @ ..] => Some(((*val).into(), xs)),
            [hb, lb, xs @ ..] => {
                let val = u16::from_le_bytes([*lb, *hb & 0x7F]);
                Some((val, xs))
            }
            _ => None,
        }
    }
}
