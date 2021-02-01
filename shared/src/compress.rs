use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryInto,
    io::Write,
};

use crate::RunLengthEncoded;

use super::{ADDR_SIZE, DICT_START_ADDR, LOOKUP_START, NUM_RECORD_ADDR, RECORD_START_ADDR};

#[derive(Debug)]
pub struct Compress {
    dict: BTreeMap<Vec<u8>, u16>,
    records: Vec<Vec<u16>>,
    max_record_len: u16,
}

impl Default for Compress {
    fn default() -> Self {
        Self::new()
    }
}

impl Compress {
    // This code results in a lot of casting with the intention of truncating.
    #![allow(clippy::clippy::cast_possible_truncation)]

    #[must_use]
    pub fn new() -> Self {
        Self {
            dict: (0..=255_u16).map(|b| (vec![b as u8], b as _)).collect(),
            records: Vec::new(),
            max_record_len: 0,
        }
    }

    #[must_use]
    pub fn with_dict(dict: BTreeSet<u8>) -> Self {
        Self {
            dict: dict
                .into_iter()
                .zip(0..)
                .map(|(c, i)| (vec![c], i))
                .collect(),
            records: Vec::new(),
            max_record_len: 0,
        }
    }

    pub fn add_record<T: AsRef<[u8]>>(&mut self, record: T) {
        let record = record.as_ref();

        let mut compressed = Vec::new();
        let mut cur_seq: &[u8] = &[];
        let mut cur_seq_start = 0;

        for idx in 0..record.len() {
            let new_seq = &record[cur_seq_start..=idx];

            if self.dict.contains_key(new_seq) {
                cur_seq = new_seq;
            } else {
                // Write current sequence to output.
                compressed.push(self.dict[cur_seq]);

                self.dict.insert(
                    new_seq.to_vec(),
                    self.dict
                        .len()
                        .try_into()
                        .expect("Unable to encode length as u16"),
                );

                // Restart the sequence starting with the current byte.
                cur_seq_start = idx;
                cur_seq = &record[idx..=idx];
            }
        }

        // We may have left-over data in cur_seq, but we'll already have
        // seen it.
        if !cur_seq.is_empty() {
            compressed.push(self.dict[cur_seq]);
        }

        self.records.push(compressed);
        self.max_record_len = self
            .max_record_len
            .max(record.len().try_into().expect("Length too long"));
    }

    /// Stores the compressed archive into a data structure readable by `Decompress`.
    #[must_use]
    pub fn store_archive(&self) -> Vec<u8> {
        let mut archive = Vec::new();
        let records_len: u16 = self
            .records
            .len()
            .try_into()
            .expect("Records length too long");

        let dictionary_keys: BTreeMap<_, _> = self.dict.iter().map(|(k, v)| (v, k)).collect();
        let lookup_length_bytes = dictionary_keys.len() * ADDR_SIZE;
        archive.resize(lookup_length_bytes + LOOKUP_START, 0);

        // Number of records.
        archive[NUM_RECORD_ADDR].copy_from_slice(records_len.to_le_bytes().as_ref());

        let mut cur_addr: u16 = 0; // Address relative to dictionary start.
                                   // Update the dictionary start address.
        let dict_start_addr: u16 = archive.len().try_into().expect("Archive length too long");
        archive[DICT_START_ADDR].copy_from_slice(dict_start_addr.to_le_bytes().as_ref());

        for (idx, val) in &dictionary_keys {
            let idx = **idx as usize;

            // Update the dictionary lookup with cur_addr.
            archive[LOOKUP_START + idx * ADDR_SIZE..][..ADDR_SIZE]
                .copy_from_slice(cur_addr.to_le_bytes().as_ref());

            // Write the length of dictionary entry (u16)
            let entry_len: u16 = val
                .len()
                .try_into()
                .expect("Dictionary entry length too long");
            archive.write_all(entry_len.to_le_bytes().as_ref()).unwrap();

            // Write the dictionary contents.
            archive.write_all(&val).unwrap();

            // Update cur_addr to start of new entry.
            cur_addr += ADDR_SIZE as u16 + entry_len;
        }

        // Update the record start address.
        let record_start_addr: u16 = archive.len().try_into().expect("Record length too long");
        archive[RECORD_START_ADDR].copy_from_slice(record_start_addr.to_le_bytes().as_ref());

        for record in &self.records {
            // Write the length of the record.
            let cur_record_len = record.len().try_into().expect("Record len too big");
            archive
                .write_all(RunLengthEncoded::encode(cur_record_len).as_ref())
                .unwrap();

            // Write the record.
            for &val in record {
                archive
                    .write_all(RunLengthEncoded::encode(val).as_ref())
                    .unwrap();
            }
        }

        archive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_single_record() {
        let mut archive = Compress::new();
        archive.add_record("TOBEORNOTTOBEORTOBEORNOT");

        let expected = vec![vec![
            84, 79, 66, 69, 79, 82, 78, 79, 84, 256, 258, 260, 265, 259, 261, 263,
        ]];

        assert_eq!(archive.records, expected);
    }

    #[test]
    fn compress_duplicate_record() {
        let mut archive = Compress::new();
        archive.add_record("TOBEORNOTTOBEORTOBEORNOT");
        archive.add_record("TOBEORNOTTOBEORTOBEORNOT");

        let expected = vec![
            vec![
                84, 79, 66, 69, 79, 82, 78, 79, 84, 256, 258, 260, 265, 259, 261, 263,
            ],
            vec![268, 260, 262, 264, 257, 269, 271, 270, 84],
        ];

        assert_eq!(archive.records, expected);
    }

    #[test]
    fn archive_round_trip() {
        let input_text = "Hello World!";
        let mut archive = Compress::with_dict(input_text.bytes().collect());

        archive.add_record(input_text);
        archive.add_record(input_text);

        let output = archive.store_archive();

        let mut reader = crate::decompress::Decompress::open(&output);
        assert_eq!(reader.num_records(), 2);

        let mut buf = [0u8; 50];
        for _ in 0..reader.num_records() {
            let record = reader.next_record(&mut buf);
            assert_eq!(record, Some(input_text.as_bytes()));
        }
    }
}
