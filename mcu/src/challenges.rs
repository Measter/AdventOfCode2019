use core::{fmt::Write, time::Duration};

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
mod intcode;

type Interface = I2CInterface<I2c<I2C1, (PB6<AF4>, PB7<AF4>)>>;
type Terminal = TerminalMode<Interface, DisplaySize128x64>;

const CHALLENGES: &[fn(&RTC, &mut Terminal) -> Duration] = &[day1::run, day2::run];

pub fn run(delayer: &mut Delay, rtc: &RTC, display: &mut Terminal) {
    let mut elapsed = Duration::default();
    for challenge in CHALLENGES {
        let _ = display.clear();
        let _ = display.write_str("    AoC 2019\r\n\n");

        elapsed += challenge(rtc, display);
        let _ = write!(display, "T. Time:{:?}", elapsed);

        // Can't do a delay of greater than 349ms. Nice job...
        delayer.delay_ms(250_u16);
        delayer.delay_ms(250_u16);
    }
}
