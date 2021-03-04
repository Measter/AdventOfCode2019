use core::{fmt::Write, time::Duration};

use numtoa::NumToA;
use shared::Reader;

use super::Terminal;
use crate::rtc::RTC;

fn is_valid(password: u32) -> (bool, bool) {
    if password < 100_000 || password > 999_999 {
        return (false, false);
    }

    let mut digits = [0; 6];
    password.numtoa(10, &mut digits);

    let mut p1_has_double = false;
    let mut p2_has_double = false;
    let mut never_decrease = true;

    for i in 1..digits.len() {
        /// wut...
        let pair_eq = digits[i - 1] == digits[i];
        let prec_eq = digits.get(i.wrapping_sub(2)) == Some(&digits[i - 1]);
        let post_eq = digits.get(i + 1) == Some(&digits[i]);

        p1_has_double |= pair_eq;
        never_decrease &= digits[i - 1] <= digits[i];

        p2_has_double |= pair_eq & !(prec_eq | post_eq);
    }

    let valid_p1 = p1_has_double && never_decrease;
    let valid_p2 = p2_has_double && never_decrease;

    (valid_p1, valid_p2)
}

pub fn run(rtc: &RTC, display: &mut Terminal) -> Duration {
    let start = rtc.now();

    let mut input = Reader::open(include_bytes!("../../../inputs/aoc_1904.bin")).unwrap();

    let mut buf = [0; 16];
    // Only one record.
    let record = input.next_record(&mut buf).unwrap().unwrap();
    let input = core::str::from_utf8(record).unwrap();

    let (begin, end) = input.trim().split_once('-').unwrap();
    let begin: u32 = begin.parse().unwrap();
    let end: u32 = end.parse().unwrap();

    let mut num_valid_p1 = 0;
    let mut num_valid_p2 = 0;

    for p in begin..=end {
        let (p1, p2) = is_valid(p);
        num_valid_p1 += p1 as u32;
        num_valid_p2 += p2 as u32;
    }

    let duration = rtc.now().elapsed_since(&start);
    let _ = writeln!(display, "Day 4:{:?}", duration);
    let _ = writeln!(display, "P1:{}", num_valid_p1);
    let _ = writeln!(display, "P2:{}", num_valid_p2);

    duration
}
