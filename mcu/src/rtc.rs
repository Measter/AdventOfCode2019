use stm32f3_discovery::stm32f3xx_hal::stm32::{
    self,
    rtc::{ssr::R as SSR, tr::R as TR},
};

const PREDIV_S: u16 = 1999;
const PREDIV_PER_SECOND: u16 = 10000 / (PREDIV_S + 1);

pub struct RTC(stm32::RTC);

impl RTC {
    pub fn init(rtc: stm32::RTC) -> Self {
        let rcc = unsafe { &*stm32::RCC::ptr() };
        let pwr = unsafe { &*stm32::PWR::ptr() };
        // Need to enable the Power system clock.
        rcc.apb1enr.modify(|_, w| w.pwren().enabled());

        // Disable RTC domain write protection.
        pwr.cr.modify(|_, w| w.dbp().set_bit());
        // Apparently we need to wait, which the datasheet doesn't say!
        // while pwr.cr.read().dbp().bit_is_clear() {}

        rcc.bdcr.modify(|_, w| w.bdrst().set_bit());
        rcc.bdcr.modify(|_, w| {
            w.bdrst().clear_bit();
            w.rtcsel().hse();
            w.rtcen().enabled()
        });

        // Disable write protection.
        rtc.wpr.write(|w| unsafe { w.key().bits(0xCA) });
        rtc.wpr.write(|w| unsafe { w.key().bits(0x53) });
        // Enter init mode.
        rtc.isr.modify(|_, w| w.init().set_bit());
        // Wait for confirmation.
        while rtc.isr.read().initf().bit_is_clear() {}

        // Prescale values
        rtc.prer.modify(|_, w| unsafe { w.prediv_a().bits(124) });
        rtc.prer
            .modify(|_, w| unsafe { w.prediv_s().bits(PREDIV_S) });

        // Set date format.
        rtc.cr.modify(|_, w| w.fmt().set_bit());

        // Reset time and date.
        rtc.dr.reset();
        rtc.tr.reset();

        // Exit init mode.
        rtc.isr.modify(|_, w| w.init().clear_bit());
        // Enable write protection
        rtc.wpr.write(|w| unsafe { w.key().bits(0xFF) });

        // Enable RTC domain write protection.
        pwr.cr.modify(|_, w| w.dbp().clear_bit());

        Self(rtc)
    }

    pub fn now(&self) -> Instant {
        let ret = Instant {
            ssr: self.0.ssr.read(),
            tr: self.0.tr.read(),
        };

        // Need to read the date to unlock the registers.
        let _ = self.0.dr.read();

        ret
    }
}

pub struct Instant {
    tr: TR,
    ssr: SSR,
}

impl core::fmt::Display for Instant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let hours = self.tr.ht().bits() * 10 + self.tr.hu().bits();
        let minutes = self.tr.mnt().bits() * 10 + self.tr.mnu().bits();
        let seconds = self.tr.st().bits() * 10 + self.tr.su().bits();

        let subsecs = (PREDIV_S - self.ssr.bits() as u16) * PREDIV_PER_SECOND;
        write!(
            f,
            "{:02}:{:02}:{:02}.{:04}",
            hours, minutes, seconds, subsecs
        )
    }
}

impl Instant {
    pub fn elapsed_since(&self, start: &Instant) -> Duration {
        let hours = (start.tr.ht().bits() * 10 + start.tr.hu().bits()) as u64;
        let minutes = (start.tr.mnt().bits() * 10 + start.tr.mnu().bits()) as u64;
        let seconds = (start.tr.st().bits() * 10 + start.tr.su().bits()) as u64;

        let start_seconds = hours * 3600 + minutes * 60 + seconds;
        let start_subsecs = (PREDIV_S - start.ssr.bits() as u16) * PREDIV_PER_SECOND;

        let hours = (self.tr.ht().bits() * 10 + self.tr.hu().bits()) as u64;
        let minutes = (self.tr.mnt().bits() * 10 + self.tr.mnu().bits()) as u64;
        let seconds = (self.tr.st().bits() * 10 + self.tr.su().bits()) as u64;

        let self_seconds = hours * 3600 + minutes * 60 + seconds;
        let self_subsecs = (PREDIV_S - self.ssr.bits() as u16) * PREDIV_PER_SECOND;

        let (overflow, subsecs) = self_subsecs
            .checked_sub(start_subsecs)
            .map(|s| (false, s))
            .unwrap_or((true, 10_000 - (start_subsecs - self_subsecs)));
        let seconds = self_seconds - start_seconds - overflow as u64;

        Duration { seconds, subsecs }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    seconds: u64,
    subsecs: u16,
}

impl Duration {
    pub fn new(seconds: u64, millis: u16) -> Self {
        Self {
            seconds,
            subsecs: ((millis % 1000) * 10) as u16,
        }
    }

    pub fn from_secs(seconds: u64) -> Self {
        Self {
            seconds,
            subsecs: 0,
        }
    }

    pub fn from_millis(millis: u64) -> Self {
        Self {
            seconds: millis / 1000,
            subsecs: ((millis % 1000) * 10) as u16,
        }
    }
}

impl core::fmt::Display for Duration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self {
                seconds: 0,
                subsecs,
            } => {
                write!(f, "{}ms", subsecs / 10)
            }
            _ => {
                write!(f, "{}.{:03}s", self.seconds, self.subsecs / 10)
            }
        }
    }
}
