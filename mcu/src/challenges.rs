use core::{
    fmt::{Display, Write},
    time::Duration,
};

use crate::rtc::RTC;
use ssd1306::{displaysize::DisplaySize128x64, mode::TerminalMode, prelude::I2CInterface};
use stm32f3_discovery::stm32f3xx_hal::{
    delay::Delay,
    gpio::{
        gpiob::{PB6, PB7},
        AF4,
    },
    i2c::I2c,
    pac::I2C1,
    prelude::_embedded_hal_blocking_delay_DelayMs,
};

mod day1;
mod day2;
mod day3;
mod day4;
mod intcode;

type Interface = I2CInterface<I2c<I2C1, (PB6<AF4>, PB7<AF4>)>>;
type Terminal = TerminalMode<Interface, DisplaySize128x64>;

const CHALLENGES: &[(u8, fn(&RTC) -> ChallengeResponse)] = &[
    (1, day1::run),
    (2, day2::run),
    (3, day3::run),
    (4, day4::run),
];

pub struct ChallengeResponse {
    pub duration: Duration,
    pub part1: Option<u64>,
    pub part2: Option<u64>,
}

pub fn run(delayer: &mut Delay, rtc: &RTC, display: &mut Terminal) {
    let mut elapsed = Duration::default();
    let _ = display.clear();
    let _ = display.write_str("    AoC 2019\r\n\n");

    for (i, challenge) in CHALLENGES {
        let _ = write!(display, "Day {}", i);
        let ChallengeResponse {
            duration,
            part1,
            part2,
        } = challenge(rtc);

        elapsed += duration;
        let _ = writeln!(display, ":{:?}", duration);

        let p1: &dyn Display = part1.as_ref().map(|i| i as _).unwrap_or(&"N/I" as _);
        let p2: &dyn Display = part2.as_ref().map(|i| i as _).unwrap_or(&"N/I" as _);

        let _ = writeln!(display, "P1:{}", p1);
        let _ = writeln!(display, "P2:{}", p2);

        // Can't do a delay of greater than 262ms. Nice job...
        delayer.delay_ms(250_u16);
        let _ = writeln!(display);
    }
    let _ = write!(display, "T. Time:{:?}", elapsed);
}
