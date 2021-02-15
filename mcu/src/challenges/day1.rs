use core::{fmt::Write, time::Duration};

use super::Terminal;
use crate::rtc::RTC;

use shared::Reader;

pub fn run(rtc: &RTC, display: &mut Terminal) -> Duration {
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
    let _ = writeln!(display, "Day 1:{:?}", duration);
    let _ = writeln!(display, "P1:{}", sum_p1);
    let _ = writeln!(display, "P2:{}\r\n", sum_p2);

    duration
}
