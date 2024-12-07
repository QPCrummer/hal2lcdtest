//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use core::time::Duration;
use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use lcd1602_rs::LCD1602;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use cortex_m::delay::Delay;
use cortex_m::peripheral::syst::SystClkSource;
use embedded_hal::timer::{Cancel, CountDown, Periodic};
use rp_pico::hal;
use nb;
use rp_pico::hal::Clock;
use void::Void;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let delay = Delay::new(core.SYST, 133000000);
    let timer: Timer = Timer::new(delay);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Init pins
    let rs = pins.gpio0.into_push_pull_output();
    let en = pins.gpio1.into_push_pull_output();
    let d4 = pins.gpio2.into_push_pull_output();
    let d5 = pins.gpio3.into_push_pull_output();
    let d6 = pins.gpio4.into_push_pull_output();
    let d7 = pins.gpio5.into_push_pull_output();

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    //
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead.
    // One way to do that is by using [embassy](https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/wifi_blinky.rs)
    //
    // If you have a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here. Don't forget adding an appropriate resistor
    // in series with the LED.
    let mut lcd = LCD1602::new(en, rs, d4, d5, d6, d7, timer).unwrap();

    loop {
        lcd.print("hello world!").ok();
        lcd.delay(1_000_000u64).ok();
        lcd.clear().ok();
        lcd.delay(1_000_000u64).ok();
    }
}

/// A simple Timer struct
pub struct Timer {
    duration: Duration,
    periodic: bool,
    delay: Delay,
}

impl Timer {
    /// Creates a new Timer
    pub fn new(delay: Delay) -> Self {
        Timer {
            duration: Duration::from_secs(0),
            periodic: false,
            delay,
        }
    }
}

impl CountDown for Timer {
    type Time = Duration;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        self.duration = count.into();
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        self.delay.delay_us(self.duration.as_micros() as u32);
        Ok(())
    }
}

impl Periodic for Timer {}

impl Cancel for Timer {
    type Error = &'static str;

    fn cancel(&mut self) -> Result<(), Self::Error> {
      Ok(())
    }
}

// End of file
