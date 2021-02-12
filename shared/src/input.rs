#[cfg(any(feature = "std", test))]
use std::convert::TryInto;

#[cfg(any(feature = "std", test))]
use crate::compress::Compress;

use crate::{decompress::Decompress, ErrorKind, RunLengthEncoded};

const COMPRESSED: u8 = 1;
const RAW: u8 = 0;

pub enum Reader<'a> {
    Compressed(Decompress<'a>),
    Raw(Raw<'a>),
}

impl<'a> Reader<'a> {
    /// # Errors
    ///
    /// Will return an error if the length of `input` is 0, the first bit is not `RAW` or `COMPRESSED`,
    /// or if `Compressed::open` or `Raw::open` fail.
    pub fn open<T: AsRef<[u8]> + ?Sized + 'a>(input: &'a T) -> Result<Reader, ErrorKind> {
        let input = input.as_ref();
        let (is_compressed, data) = input
            .split_first()
            .ok_or(ErrorKind::InvalidCompressedFlag)?;

        match *is_compressed {
            RAW => Ok(Reader::Raw(Raw::open(data)?)),
            COMPRESSED => Ok(Reader::Compressed(Decompress::open(data))),
            _ => Err(ErrorKind::InvalidCompressedFlag),
        }
    }

    #[must_use]
    pub fn num_records(&self) -> usize {
        match self {
            Self::Compressed(c) => c.num_records(),
            Self::Raw(r) => r.num_records(),
        }
    }

    /// Reads the next record from the file.
    ///
    /// # Errors
    ///
    /// Returns an error:
    ///
    /// * On failure to read record length.
    /// * Record length exceeds output buffer length.
    /// * Record length exceeds remaining file length.
    pub fn next_record<'b>(&mut self, dst: &'b mut [u8]) -> Result<Option<&'b [u8]>, ErrorKind> {
        match self {
            Self::Compressed(c) => c.next_record(dst),
            Self::Raw(r) => r.next_record(dst),
        }
    }
}

pub struct Raw<'a> {
    num_records: usize,
    records: &'a [u8],
    current_record: usize,
}

impl<'a> Raw<'a> {
    fn open<T: AsRef<[u8]> + ?Sized + 'a>(data: &'a T) -> Result<Self, ErrorKind> {
        let data = data.as_ref();
        let (num_records, records) =
            RunLengthEncoded::decode(data).ok_or(ErrorKind::LengthDecode)?;

        Ok(Self {
            num_records: num_records as usize,
            records,
            current_record: 0,
        })
    }

    #[must_use]
    pub fn num_records(&self) -> usize {
        self.num_records
    }

    /// Reads the next record from the file.
    ///
    /// # Errors
    ///
    /// Returns an error:
    ///
    /// * On failure to read record length.
    /// * Record length exceeds output buffer length.
    /// * Record length exceeds remaining file length.
    pub fn next_record<'b>(&mut self, dst: &'b mut [u8]) -> Result<Option<&'b [u8]>, ErrorKind> {
        if self.current_record == self.records.len() {
            return Ok(None);
        }

        let (len, remaining_bytes) = RunLengthEncoded::decode(&self.records[self.current_record..])
            .ok_or(ErrorKind::RecordReadError)?;
        let len = len as usize;

        if dst.len() < len {
            return Err(ErrorKind::RecordReadError);
        }
        let record = remaining_bytes
            .get(..len)
            .ok_or(ErrorKind::RecordReadError)?;

        dst.copy_from_slice(record);
        let written_buf = &dst[..len];

        let len_dif =
            self.records.len() - self.current_record - (remaining_bytes.len() - record.len());
        self.current_record += len_dif;

        Ok(Some(written_buf))
    }
}

#[cfg(any(feature = "std", test))]
#[derive(Default)]
pub struct Writer<'a> {
    raw_records: Vec<&'a [u8]>,
    compressor: Compress,
}

#[cfg(any(feature = "std", test))]
impl<'a> Writer<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            raw_records: Vec::new(),
            compressor: Compress::new(),
        }
    }

    pub fn preload_dict<T: AsRef<[u8]> + ?Sized + 'a>(&mut self, entries: &[&T]) {
        entries
            .iter()
            .for_each(|e| self.compressor.add_dictionary_entry(e))
    }

    pub fn add_record<T: AsRef<[u8]> + ?Sized + 'a>(&mut self, record: &'a T) {
        self.compressor.add_record(record);
        self.raw_records.push(record.as_ref());
    }

    /// # Errors
    ///
    /// Returns error on failure to write to the `writer`.
    pub fn write(&self, mut writer: impl std::io::Write) -> Result<(), ErrorKind> {
        let mut raw = Vec::with_capacity(self.raw_records.iter().map(|r| r.len() + 2).sum());

        self.raw_records.iter().for_each(|r| {
            raw.extend_from_slice(RunLengthEncoded::encode(r.len().try_into().unwrap()).as_ref());
            raw.extend_from_slice(r);
        });

        let compressed = self.compressor.store_archive();

        if compressed.len() > raw.len() {
            writer.write_all(&[COMPRESSED])?;
            writer.write_all(&compressed)?;
        } else {
            writer.write_all(&[RAW])?;
            writer.write_all(&raw)?;
        }

        Ok(())
    }
}
