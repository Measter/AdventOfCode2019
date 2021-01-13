#![no_std]
#![no_main]

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use panic_semihosting as _;
use ssd1306::{
    displaysize::DisplaySize128x32, mode::TerminalMode, Builder as DisplayBuilder, I2CDIBuilder,
};
use stm32f3_discovery::stm32f3xx_hal::{
    i2c::I2c,
    prelude::*,
    stm32,
    usb::{Peripheral as USBPeripheral, UsbBus},
};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use core::fmt::Write;

#[entry]
fn main() -> ! {
    let periphs = stm32::Peripherals::take().unwrap();
    let mut rcc = periphs.RCC.constrain();

    let mut flash = periphs.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    // Initialize USB serial.
    let mut gpioa = periphs.GPIOA.split(&mut rcc.ahb);
    let mut usb_dp = gpioa
        .pa12
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    usb_dp.set_low().ok();
    delay(clocks.sysclk().0 / 100); // 10ms wait.

    let usb_dm = gpioa.pa11.into_af14(&mut gpioa.moder, &mut gpioa.afrh);
    let usb_db = usb_dp.into_af14(&mut gpioa.moder, &mut gpioa.afrh);

    let usb = USBPeripheral {
        usb: periphs.USB,
        pin_dm: usb_dm,
        pin_dp: usb_db,
    };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16C0, 0x27DD))
        .product("AoC 2019")
        .device_class(USB_CLASS_CDC)
        .build();

    // Initialize display.
    let mut gpiob = periphs.GPIOB.split(&mut rcc.ahb);

    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sca = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(periphs.I2C1, (scl, sca), 400.khz(), clocks, &mut rcc.apb1);

    let interface = I2CDIBuilder::new().with_i2c_addr(0x3C).init(i2c);
    let mut display: TerminalMode<_, _> = DisplayBuilder::new()
        .size(DisplaySize128x32)
        .connect(interface)
        .into();
    display.init().unwrap();
    let _ = display.clear();

    writeln!(&mut display, "AoC 2019");

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Ok(len) => {
                let text = core::str::from_utf8(&buf[..len]).unwrap();
                let _ = display.write_str(text);
            }
            Err(_) => {}
        }
    }
}
