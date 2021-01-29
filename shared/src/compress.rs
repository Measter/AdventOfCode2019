use std::{
    collections::{BTreeMap, HashMap},
    convert::TryInto,
    io::Write,
};

const ADDR_SIZE: usize = 2;
const LOOKUP_START: usize = 6;

#[derive(Debug)]
pub struct Compress {
    dict: HashMap<Vec<u8>, u16>,
    records: Vec<Vec<u16>>,
    max_record_len: u16,
}

impl Compress {
    pub fn new() -> Self {
        Self {
            dict: (0..=255u16).map(|b| (vec![b as u8], b as _)).collect(),
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
                cur_seq = &record[idx..idx + 1];
            }
        }

        // We may have left-over data in cur_seq, but we'll already have
        // seen it.
        if !cur_seq.is_empty() {
            compressed.push(self.dict[cur_seq]);
        }

        self.records.push(compressed);
        self.max_record_len = self.max_record_len.max(record.len() as _);
    }

    /// Stores the compressed archive into a data structure readable by `Decompress`.
    pub fn store_archive(&self) -> Vec<u8> {
        let mut archive = Vec::new();

        // Address of the dictionary start.
        archive.write(0u16.to_le_bytes().as_ref()).unwrap();
        // Address of records start.
        archive.write(0u16.to_le_bytes().as_ref()).unwrap();
        // Number of records.
        archive
            .write((self.records.len() as u16).to_le_bytes().as_ref())
            .unwrap();

        let dictionary_keys: BTreeMap<_, _> = self.dict.iter().map(|(k, v)| (v, k)).collect();

        // Address size is u16 (2 bytes).
        let lookup_length_bytes = dictionary_keys.len() * ADDR_SIZE;
        archive.resize(lookup_length_bytes + LOOKUP_START, 0);
        let mut cur_addr = archive.len() as u16;
        // Copy in our dictionary address.
        archive[0..2].copy_from_slice(cur_addr.to_le_bytes().as_ref());

        for (idx, val) in dictionary_keys.iter() {
            let idx = **idx as usize;

            // Update the dictionary lookup with cur_addr.
            archive[LOOKUP_START + idx..][..ADDR_SIZE]
                .copy_from_slice(cur_addr.to_le_bytes().as_ref());

            // Write the length of dictionary entry (u16)
            archive
                .write((val.len() as u16).to_le_bytes().as_ref())
                .unwrap();

            // Write the dictionary contents.
            archive.write(&val).unwrap();

            // Update cur_addr to start of new entry.
            cur_addr += ADDR_SIZE as u16 + val.len() as u16;
        }

        // cur_addr should now be the start of our records.
        // Update the record start address.
        archive[2..4].copy_from_slice(cur_addr.to_le_bytes().as_ref());

        for record in &self.records {
            // Write the length of the record.
            archive
                .write((record.len() as u16 * 2).to_le_bytes().as_ref())
                .unwrap();

            // Write the record.
            for val in record {
                archive.write(val.to_le_bytes().as_ref()).unwrap();
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
}
