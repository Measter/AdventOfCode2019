use std::{
    fs::{read_to_string, File},
    io::ErrorKind,
};

use shared::Writer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn archive_input<F>(day: u8, dict_preload: &[&str], record_func: F) -> Result<()>
where
    F: for<'a> Fn(&mut Writer<'a>, &'a str) -> String,
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
    let msg = record_func(&mut archive, &contents);

    let output_file = File::create(format!("../inputs/aoc_19{:02}.bin", day))?;
    archive.write(output_file).unwrap();

    println!(" Written. {}", msg);

    Ok(())
}

fn by_line<'a>(archive: &mut Writer<'a>, input: &'a str) -> String {
    let max_len = input.lines().fold(0, |max_len, record| {
        archive.add_record(record);
        max_len.max(record.len())
    });

    format!("Longest record: {} bytes", max_len)
}

fn intcode_split<'a>(archive: &mut Writer<'a>, input: &'a str) -> String {
    // This one should print the maximum address used, as well as the longest record.

    let mut max_address = 0;
    let mut longest_record = 0;

    for (i, num) in input.split(',').enumerate() {
        max_address = i;
        longest_record = longest_record.max(num.trim().len());
        archive.add_record(num);
    }

    format!(
        "Max Addr: {}, Longest Record: {} bytes",
        max_address, longest_record
    )
}

fn day_3<'a>(archive: &mut Writer<'a>, input: &'a str) -> String {
    let mut lines = input.lines();
    let first_wire = lines.next().unwrap();
    let second_wire = lines.next().unwrap();

    let mut max_len = 0;
    let mut num_instrs_first_wire = 0;
    for instr in first_wire.split(',') {
        archive.add_record(instr);
        num_instrs_first_wire += 1;
        max_len = instr.len().max(max_len);
    }

    archive.add_record("-");
    let mut num_instrs_second_wire = 0;
    for instr in second_wire.split(',') {
        archive.add_record(instr);
        num_instrs_second_wire += 1;
        max_len = instr.len().max(max_len);
    }

    format!(
        "Max record: {} bytes, Wire 1 Length: {}, Wire 2 Length: {}",
        max_len, num_instrs_first_wire, num_instrs_second_wire
    )
}

fn main() -> Result<()> {
    archive_input(1, &[], by_line)?;
    archive_input(2, &["1,", "2,", "99,"], intcode_split)?;
    archive_input(3, &[], day_3)?;
    archive_input(4, &[], by_line)?;

    Ok(())
}
