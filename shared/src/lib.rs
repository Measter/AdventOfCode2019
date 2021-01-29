#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(any(feature = "std", test))]
extern crate std;

#[cfg(any(feature = "std", test))]
pub mod compress;

pub mod decompress;
