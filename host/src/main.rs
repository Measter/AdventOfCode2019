use std::{
    fs::{read_to_string, File},
    io::{ErrorKind, Write},
};

use shared::{compress::Compress, RunLengthEncoded};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn archive_input<F>(day: u8, dict_preload: &[&str], record_func: F) -> Result<()>
where
    F: Fn(&mut Compress, &str) -> (u16, usize),
{
    print!("Day {}... ", day);

    let contents = match read_to_string(format!("../inputs/aoc_19{:02}.txt", day)) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            println!("Input not found.");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let mut archive = Compress::new();
    dict_preload
        .iter()
        .for_each(|d| archive.add_dictionary_entry(d));

    let (num_records, max_len) = record_func(&mut archive, &contents);
    let compressed = archive.store_archive();

    let mut output_file = File::create(format!("../inputs/aoc_19{:02}.bin", day))?;

    if compressed.len() > contents.len() {
        println!("Not compressed.");
        output_file.write_all(&[0])?;
        output_file.write_all(RunLengthEncoded::encode(num_records).as_ref())?;
        output_file.write_all(contents.as_bytes())?;
    } else {
        println!("Compressed. Max record: {}b", max_len);
        output_file.write_all(&[1])?;
        output_file.write_all(&compressed)?;
    }

    Ok(())
}

fn day1(archive: &mut Compress, input: &str) -> (u16, usize) {
    let max_len = input.lines().fold((0, 0), |(count, max_len), record| {
        archive.add_record(record);
        (count + 1, max_len.max(record.len()))
    });

    max_len
}

fn main() -> Result<()> {
    archive_input(1, &[], day1)?;

    Ok(())
}
