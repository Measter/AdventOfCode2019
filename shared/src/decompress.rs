use crate::{RunLengthEncoded, RECORD_START_ADDR};

use super::{ADDR_SIZE, DICT_START_ADDR, LOOKUP_START, NUM_RECORD_ADDR};

#[derive(Debug)]
pub struct Decompress<'a> {
    // Stores the index into the dictionary table.
    dict_lookup: &'a [u8],

    //  Decompression dictionary.
    dict: &'a [u8],

    // Compressed records.
    records: &'a [u8],

    num_records: usize,

    // Stores the index into the `records` field above for the start of the next record.
    current_record: usize,
}

impl<'a> Decompress<'a> {
    pub fn open<T: AsRef<[u8]> + 'a>(data: &'a T) -> Self {
        let data = data.as_ref();

        let mut size_buf = [0_u8; 2];

        size_buf.copy_from_slice(&data[NUM_RECORD_ADDR]);
        let num_records = u16::from_le_bytes(size_buf) as usize;

        size_buf.copy_from_slice(&data[DICT_START_ADDR]);
        let dict_start_addr = u16::from_le_bytes(size_buf) as usize;

        size_buf.copy_from_slice(&data[RECORD_START_ADDR]);
        let record_idx_addr = u16::from_le_bytes(size_buf) as usize;

        let dict_lookup = &data[LOOKUP_START..dict_start_addr];
        let dict = &data[dict_start_addr..record_idx_addr];
        let records = &data[record_idx_addr..];

        Self {
            dict_lookup,
            dict,
            records,
            num_records,
            current_record: 0,
        }
    }

    #[must_use]
    pub fn num_records(&self) -> usize {
        self.num_records
    }

    fn dict_lookup(&self, id: u16) -> &[u8] {
        let idx = id as usize * ADDR_SIZE;

        // Decode address into dict.
        let mut buf = [0_u8; 2];
        buf.copy_from_slice(&self.dict_lookup[idx..idx + 2]);
        let addr = u16::from_le_bytes(buf) as usize;

        // Decode length of dict entry.
        buf.copy_from_slice(&self.dict[addr..addr + 2]);
        let len = u16::from_le_bytes(buf) as usize;

        // Now we finally get the slice to return.
        &self.dict[addr + 2..addr + 2 + len]
    }

    pub fn next_record<'b>(&mut self, dst: &'b mut [u8]) -> Option<&'b [u8]> {
        let (len, mut remaining_bytes) = self
            .records
            .get(self.current_record..)
            .and_then(RunLengthEncoded::decode)?;

        let mut end = 0;
        for _ in 0..len {
            let (id, rem) = RunLengthEncoded::decode(remaining_bytes)?;
            remaining_bytes = rem;

            let dict_entry = self.dict_lookup(id);

            dst[end..end + dict_entry.len()].copy_from_slice(dict_entry);
            end += dict_entry.len();
        }

        let written_buf = &dst[..end];
        let len_dif = self.records.len() - self.current_record - remaining_bytes.len();
        self.current_record += len_dif;

        Some(written_buf)
    }
}
