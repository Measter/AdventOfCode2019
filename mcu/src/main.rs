#![no_std]
#![no_main]
#![feature(str_split_once, array_windows)]

use cortex_m_rt::entry;
use panic_semihosting as _;
use ssd1306::{
    displaysize::DisplaySize128x64, mode::TerminalMode, Builder as DisplayBuilder, I2CDIBuilder,
};
use stm32f3_discovery::stm32f3xx_hal::{delay::Delay, i2c::I2c, prelude::*, stm32};

mod challenges;
mod rtc;

#[entry]
fn main() -> ! {
    let core_periphs = stm32::CorePeripherals::take().unwrap();
    let periphs = stm32::Peripherals::take().unwrap();

    let mut rcc = periphs.RCC.constrain();

    let mut flash = periphs.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(64.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr);

    // Initialize display.
    let mut gpiob = periphs.GPIOB.split(&mut rcc.ahb);

    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sca = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(periphs.I2C1, (scl, sca), 400.khz(), clocks, &mut rcc.apb1);

    let interface = I2CDIBuilder::new().with_i2c_addr(0x3C).init(i2c);
    let mut display: TerminalMode<_, _> = DisplayBuilder::new()
        .size(DisplaySize128x64)
        .connect(interface)
        .into();
    display.init().unwrap();
    let _ = display.clear();

    let rtc = rtc::RTC::init(periphs.RTC);
    let mut delayer = Delay::new(core_periphs.SYST, clocks);

    challenges::run(&mut delayer, &rtc, &mut display);

    loop {}
}
