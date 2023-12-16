#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use display_interface::DisplayError;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config, InterruptHandler as I2cInterruptHandler};
use embassy_rp::peripherals::{I2C1, USB};
use embassy_rp::usb::{Driver, InterruptHandler as USBInterruptHandler};
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => USBInterruptHandler<USB>;
    I2C1_IRQ => I2cInterruptHandler<I2C1>;
});

#[embassy_executor::task]
async fn usb_logger_task(usb_driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, usb_driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Init USB logger
    let usb_driver = Driver::new(p.USB, Irqs);
    spawner.spawn(usb_logger_task(usb_driver)).unwrap();

    // Init I2C display
    let sda = p.PIN_26;
    let scl = p.PIN_27;
    let i2c = i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());
    let interface = I2CDisplayInterface::new(i2c);
    let mut display =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
    let display_init_res = match display.init() {
        Ok(_) => "ok",
        Err(e) => match e {
            DisplayError::BusWriteError => "bus write error",
            _ => "other",
        },
    };

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Aguante Rust", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    let delay = Duration::from_secs(1);
    loop {
        log::info!("{display_init_res}");
        Timer::after(delay).await;
    }
}
