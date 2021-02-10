#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![warn(clippy::pedantic)]

use core::convert::TryInto;

#[cfg(any(feature = "std", test))]
extern crate std;

#[cfg(any(feature = "std", test))]
pub mod compress;

pub mod decompress;

const ADDR_SIZE: usize = 2;
const DICT_START_ADDR: core::ops::Range<usize> = 0..2;
const RECORD_START_ADDR: core::ops::Range<usize> = 2..4;
const NUM_RECORD_ADDR: core::ops::Range<usize> = 4..6;
const LOOKUP_START: usize = 6;

const RUN_LEN_MAX_BYTE: u16 = 0x7F;
const MULTI_BYTE_START: u8 = 0x80;

#[derive(Debug, Copy, Clone)]
pub enum RunLengthEncoded {
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
    pub fn encode(val: u16) -> RunLengthEncoded {
        if val > RUN_LEN_MAX_BYTE {
            let low_byte: u8 = (val & RUN_LEN_MAX_BYTE).try_into().unwrap();
            let low_byte = low_byte | MULTI_BYTE_START;
            let high_byte = ((val >> 7) & 0xFF).try_into().unwrap();

            RunLengthEncoded::Double([low_byte, high_byte])
        } else {
            #[allow(clippy::clippy::cast_possible_truncation)]
            RunLengthEncoded::Single([val as u8])
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(u16, &[u8])> {
        match bytes {
            [val @ 0..=0x7F, xs @ ..] => Some(((*val).into(), xs)),
            [low_byte @ 0x80..=0xFF, high_byte, xs @ ..] => {
                let upper: u16 = (*high_byte).into();
                let lower: u16 = (*low_byte).into();
                Some(((upper << 7) | lower & RUN_LEN_MAX_BYTE, xs))
            }
            _ => None,
        }
    }
}
