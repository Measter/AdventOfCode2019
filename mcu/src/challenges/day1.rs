use core::{fmt::Write, time::Duration};

use crate::rtc::RTC;

use shared::Reader;
use ssd1306::{
    mode::{terminal::TerminalDisplaySize, TerminalMode},
    prelude::WriteOnlyDataCommand,
};

pub fn run<T, S>(rtc: &RTC, display: &mut TerminalMode<T, S>) -> Duration
where
    T: WriteOnlyDataCommand,
    S: TerminalDisplaySize,
{
    let _ = display.clear();
    let _ = display.write_str("AoC 2019 Day 1\r\n");

    let mut input = Reader::open(include_bytes!("../../../inputs/aoc_1901.bin")).unwrap();

    // Part 1
    let start = rtc.now();

    let mut sum_p1 = 0;
    let mut sum_p2 = 0;
    let mut buf = [0; 6];
    while let Some(record) = input.next_record(&mut buf).unwrap() {
        let mass: u64 = core::str::from_utf8(record).unwrap().parse().unwrap();
        sum_p1 += mass / 3 - 2;

        let mut new_fuel = 0;
        let mut cur_mass = mass;
        loop {
            match (cur_mass / 3).checked_sub(2) {
                Some(0) | None => break,
                Some(f) => {
                    new_fuel += f;
                    cur_mass = f;
                }
            }
        }
        sum_p2 += new_fuel;
    }

    let duration = rtc.now().elapsed_since(&start);
    let _ = writeln!(display, "Part 1:\n\r{}", sum_p1);
    let _ = writeln!(display, "Part 2:\n\r{}", sum_p2);

    duration
}
