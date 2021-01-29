#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![warn(clippy::pedantic)]

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
