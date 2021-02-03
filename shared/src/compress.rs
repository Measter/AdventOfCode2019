use std::{collections::BTreeMap, convert::TryInto, io::Write};

use crate::{
    RunLengthEncoded, ADDR_SIZE, DICT_START_ADDR, LOOKUP_START, NUM_RECORD_ADDR, RECORD_START_ADDR,
};

const MAX_DICT_LEN: usize = 512;
const MAX_DICT_ENTRY_LEN: usize = 10;

// Applies a 3-stage compression:
//
// * Stage 1
//   Uses an unrestricted dictionary.
//   Only reads and stores the inputs, doesn't compress them.
//
// * Stage 2
//   Re-uses the stage 1 dictionary to count how many times each sequence
//   is found. Input is compressed at this stage.
//
// * Stage 3
//   Uses the counts from stage 2 to build a dictionary consisting of the
//   byte sequences seen.
//   Then re-maps the compressed records from stage 2 to the new dictionary.
#[derive(Debug)]
pub struct Compress {
    // Stage 0 dictionary.
    dict: BTreeMap<Vec<u8>, u32>,
    // Uncompressed inputs.
    records: Vec<Vec<u8>>,
    max_record_len: u32,
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
            dict: (0..=255_u32).map(|b| (vec![b as u8], b as _)).collect(),
            records: Vec::new(),
            max_record_len: 0,
        }
    }

    pub fn add_record<T: AsRef<[u8]>>(&mut self, record: T) {
        // Applies stage 1 of the process above.
        // We only iterate over the input, not compress it here.
        let record = record.as_ref();

        let mut cur_seq_start = 0;

        for idx in 0..record.len() {
            let new_seq = &record[cur_seq_start..=idx];

            if !self.dict.contains_key(new_seq) || new_seq.len() >= MAX_DICT_ENTRY_LEN {
                // A bit odd, but we don't want to remove the previous entry.
                if !self.dict.contains_key(new_seq) {
                    let val = self.dict.insert(
                        new_seq.to_vec(),
                        self.dict
                            .len()
                            .try_into()
                            .expect("Unable to encode length as u32"),
                    );

                    if val.is_some() {
                        panic!("Incorrectly removed previous entry");
                    }
                }

                // Restart the sequence starting with the current byte.
                cur_seq_start = idx;
            }
        }

        self.max_record_len = self
            .max_record_len
            .max(record.len().try_into().expect("Length too long"));

        // Finally store the input for compression later.
        self.records.push(record.to_owned());
    }

    fn apply_stage2(&self) -> (BTreeMap<u32, u32>, Vec<Vec<u32>>) {
        // Iterates over the inputs a second time, counting how many times each
        // sequences was seen while we compress.
        // This is, essentially, re-applying what we did in stage 1, but without modifying
        // the dictionary.

        let mut counts = BTreeMap::new();
        let mut compressed = Vec::new();

        for record in &self.records {
            let mut cur_compressed = Vec::new();
            let mut cur_seq: &[u8] = &[];
            let mut cur_seq_start = 0;

            for idx in 0..record.len() {
                let new_seq = &record[cur_seq_start..=idx];

                if self.dict.contains_key(new_seq) {
                    cur_seq = new_seq;
                } else {
                    cur_compressed.push(self.dict[cur_seq]);
                    *counts.entry(self.dict[cur_seq]).or_default() += 1;

                    // Restart the sequence from the current byte.
                    cur_seq_start = idx;
                    cur_seq = &record[idx..=idx];
                }
            }

            // We make have left-over data, but it's already been seen.
            if !cur_seq.is_empty() {
                cur_compressed.push(self.dict[cur_seq]);
                *counts.entry(self.dict[cur_seq]).or_default() += 1;
            }

            compressed.push(cur_compressed);
        }

        (counts, compressed)
    }

    fn apply_stage3(
        &self,
        counts: BTreeMap<u32, u32>,
        compressed: Vec<Vec<u32>>,
    ) -> (BTreeMap<Vec<u8>, u16>, Vec<Vec<u16>>) {
        // Here we build up the final dictionary and reprocess the compressed entries.
        // We start by building the final dictionary. While we do that, we build a map
        // between the stage 1 dictionary and the final dictionary.

        let id_to_seq: BTreeMap<_, _> = self.dict.iter().map(|(a, b)| (b, a)).collect();
        let mut id_to_new_id: BTreeMap<u32, u16> = BTreeMap::new();
        let mut final_dict: BTreeMap<Vec<u8>, u16> = BTreeMap::new();

        let single_bytes_seen = counts.into_iter().filter(|(_, times_seen)| *times_seen > 0);
        for (id, _) in single_bytes_seen {
            let new_id: u16 = final_dict
                .len()
                .try_into()
                .expect("Unable to encode length as u16");
            final_dict.insert(id_to_seq[&id].to_vec(), new_id);
            id_to_new_id.insert(id, new_id);
        }

        // Now we need to re-map the compressed records to the new IDs.
        let final_compressed = compressed
            .into_iter()
            .map(|record| record.into_iter().map(|id| id_to_new_id[&id]).collect())
            .collect();

        (final_dict, final_compressed)
    }

    /// Stores the compressed archive into a data structure readable by `Decompress`.
    #[must_use]
    pub fn store_archive(&self) -> Vec<u8> {
        let (stage2_counts, compressed) = self.apply_stage2();
        let (final_dict, compressed_records) = self.apply_stage3(stage2_counts, compressed);

        let mut archive = Vec::new();
        let records_len: u16 = compressed_records
            .len()
            .try_into()
            .expect("Records length too long");

        let dictionary_keys: BTreeMap<_, _> = final_dict.iter().map(|(k, v)| (v, k)).collect();
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

        for record in compressed_records {
            // Write the length of the record.
            let cur_record_len = record.len().try_into().expect("Record len too big");
            archive
                .write_all(RunLengthEncoded::encode(cur_record_len).as_ref())
                .unwrap();

            // Write the record.
            for val in record {
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
    fn stage1() {
        let input = "Hello World!";

        let mut archive = Compress::new();
        archive.add_record(input);

        let expected_record = vec![input.as_bytes().to_vec()];
        let mut expected_dict: BTreeMap<_, _> = (0..=255_u8).map(|b| (vec![b], b.into())).collect();

        let len_dict: u32 = expected_dict.len().try_into().unwrap();
        expected_dict.extend(
            input
                .as_bytes()
                .windows(2)
                .zip(len_dict..)
                .map(|(bytes, id)| (bytes.to_vec(), id)),
        );

        assert_eq!(expected_record, archive.records);
        assert_eq!(expected_dict, archive.dict);
    }

    #[test]
    fn stage2() {
        let input = "Hello World!";

        let mut archive = Compress::new();
        archive.add_record(input);

        let (actual_stage2, stage2_compressed) = archive.apply_stage2();

        let mut expected_counts = BTreeMap::new();
        expected_counts.extend((256..).step_by(2).map(|id| (id, 1)).take(6));
        assert_eq!(expected_counts, actual_stage2);

        let expected_compressed: Vec<Vec<u32>> = vec![vec![256, 258, 260, 262, 264, 266]];
        assert_eq!(expected_compressed, stage2_compressed);
    }

    #[test]
    fn stage3() {
        let input = "Hello World!";

        let mut archive = Compress::new();
        archive.add_record(input);

        let (stage2_count, stage2_compressed) = archive.apply_stage2();
        let (stage3_dict, compressed) = archive.apply_stage3(stage2_count, stage2_compressed);

        let expected_dict: BTreeMap<_, _> = [
            (vec![72, 101], 0),
            (vec![87, 111], 3),
            (vec![100, 33], 5),
            (vec![108, 108], 1),
            (vec![111, 32], 2),
            (vec![114, 108], 4),
        ]
        .iter()
        .cloned()
        .collect();
        let expected_compressed: Vec<Vec<u16>> = vec![vec![0, 1, 2, 3, 4, 5]];

        assert_eq!(expected_dict, stage3_dict);
        assert_eq!(expected_compressed, compressed);
    }

    #[test]
    fn compress_single_record() {
        let mut archive = Compress::new();
        archive.add_record("TOBEORNOTTOBEORTOBEORNOT");
        let (stage2_counts, stage2_compressed) = archive.apply_stage2();
        let (_, records) = archive.apply_stage3(stage2_counts, stage2_compressed);

        let expected = vec![vec![5, 2, 3, 4, 1, 6, 5, 2, 3, 0]];

        assert_eq!(records, expected);
    }

    #[test]
    fn compress_duplicate_record() {
        let mut archive = Compress::new();
        archive.add_record("TOBEORNOTTOBEORTOBEORNOT");
        archive.add_record("TOBEORNOTTOBEORTOBEORNOT");
        let (stage2_counts, stage2_compressed) = archive.apply_stage2();
        let (_, records) = archive.apply_stage3(stage2_counts, stage2_compressed);

        let expected = vec![vec![1, 0, 1, 1, 0], vec![1, 0, 1, 1, 0]];

        assert_eq!(records, expected);
    }

    #[test]
    fn archive_round_trip() {
        let input_text = "Hello World!";
        let mut archive = Compress::new();

        archive.add_record(input_text);
        archive.add_record(input_text);

        let output = archive.store_archive();

        let mut reader = crate::decompress::Decompress::open(&output);
        assert_eq!(reader.num_records(), 2);

        let mut buf = [0_u8; 50];
        for _ in 0..reader.num_records() {
            let record = reader.next_record(&mut buf);
            assert_eq!(record, Some(input_text.as_bytes()));
        }
    }

    #[test]
    fn big_file_test() {
        let input_text = std::fs::read_to_string("test_data/aoc_2002.txt").unwrap();
        let mut archive = Compress::new();

        let mut num_records = 0;
        for line in input_text.lines() {
            archive.add_record(line);
            num_records += 1;
        }

        let output = archive.store_archive();

        let mut reader = crate::decompress::Decompress::open(&output);
        assert_eq!(num_records, reader.num_records());

        let mut buf = [0_u8; 50];
        for (i, line) in input_text.lines().enumerate() {
            let record = reader.next_record(&mut buf);
            assert_eq!(record, Some(line.as_bytes()), "{}: {}", i, line);
        }
    }
}
