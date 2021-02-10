use core::{fmt::Write, time::Duration};

use crate::rtc::RTC;

use ssd1306::{
    mode::{terminal::TerminalDisplaySize, TerminalMode},
    prelude::WriteOnlyDataCommand,
};

pub fn run<T, S>(rtc: &RTC, display: &mut TerminalMode<T, S>) -> Duration
where
    T: WriteOnlyDataCommand,
    S: TerminalDisplaySize,
{
    let _ = display.write_str("Day 1, Part 1");

    Duration::default()
}
