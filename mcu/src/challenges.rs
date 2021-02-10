use core::{fmt::Write, time::Duration};

use crate::rtc::RTC;
use ssd1306::{
    mode::{terminal::TerminalDisplaySize, TerminalMode},
    prelude::WriteOnlyDataCommand,
};

mod day1;

pub fn run<T, S>(rtc: &RTC, display: &mut TerminalMode<T, S>)
where
    T: WriteOnlyDataCommand,
    S: TerminalDisplaySize,
{
    let _ = display.write_str("AoC 2019");

    let mut elapsed = Duration::default();

    elapsed += day1::run(rtc, display);
}