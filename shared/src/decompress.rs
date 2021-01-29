#[derive(Debug)]
pub struct Decompress<'a> {
    // Stores the index into the dictionary table.
    dict_lookup: &'a [u8],

    //  Decompression dictionary.
    dict: &'a [u8],

    // Compressed records.
    records: &'a [u8],

    // Stores the index into the `records` field above for the start of the next record.
    current_record: usize,
}
