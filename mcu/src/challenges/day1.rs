use core::{fmt::Write, time::Duration};

use crate::{input::Input, rtc::RTC};

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

    let input = Input::new(include_bytes!("../../../inputs/aoc_1901.bin"));
    // We expect the input to be raw.
    let input = if let Input::Raw { data, .. } = input {
        data
    } else {
        panic!("Expected day 1 input to be raw.");
    };

    let mut total_duration = Duration::default();

    // Part 1
    let _ = display.write_str("Part 1: \r\n");
    let start = rtc.now();
    let sum = input
        .lines()
        .map(str::parse::<u64>)
        .try_fold(0_u64, |sum, val| val.map(|v| (v / 3 - 2) + sum))
        .expect("Invalid input");
    total_duration += rtc.now().elapsed_since(&start);

    let _ = writeln!(display, "{}", sum);

    // Part 2
    let _ = display.write_str("Part 2:\r\n");
    let start = rtc.now();

    let sum = input
        .lines()
        .map(str::parse::<u64>)
        .try_fold(0_u64, |sum, val| {
            val.map(|v| {
                let mut new_fuel = 0;
                let mut cur_mass = v;

                loop {
                    match (cur_mass / 3).checked_sub(2) {
                        Some(0) | None => break,
                        Some(f) => {
                            new_fuel += f;
                            cur_mass = f;
                        }
                    }
                }

                new_fuel + sum
            })
        })
        .expect("Invalid input");
    let _ = writeln!(display, "{}", sum);

    total_duration += rtc.now().elapsed_since(&start);

    total_duration
}
