use std::{
    fs::{read_to_string, File},
    io::ErrorKind,
};

use shared::Writer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn archive_input<F>(day: u8, dict_preload: &[&str], record_func: F) -> Result<()>
where
    F: for<'a> Fn(&mut Writer<'a>, &'a str) -> usize,
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

    let mut archive = Writer::new();
    archive.preload_dict(dict_preload);
    let max_len = record_func(&mut archive, &contents);

    let output_file = File::create(format!("../inputs/aoc_19{:02}.bin", day))?;
    archive.write(output_file).unwrap();

    println!(" Written. Longest record: {}b", max_len);

    Ok(())
}

fn by_line<'a>(archive: &mut Writer<'a>, input: &'a str) -> usize {
    let max_len = input.lines().fold(0, |max_len, record| {
        archive.add_record(record);
        max_len.max(record.len())
    });

    max_len
}

fn main() -> Result<()> {
    archive_input(1, &[], by_line)?;

    Ok(())
}
