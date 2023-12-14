//! This example test the RP Pico W on board LED.
//!
//! It does not work with the RP Pico board. See blinky.rs.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, Config, InterruptHandler as I2cInterruptHandler};
use embassy_rp::peripherals::{DMA_CH0, I2C1, PIN_23, PIN_25, PIO0, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::usb::{Driver, InterruptHandler as USBInterruptHandler};
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use embedded_hal_async::i2c::I2c;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => USBInterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    I2C1_IRQ => I2cInterruptHandler<I2C1>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static, PIN_23>, PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

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
    // let sda = p.PIN_14;
    // let scl = p.PIN_15;
    // let mut i2c = i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());
    // let interface = I2CDisplayInterface::new(i2c);
    // let mut display =
    //     Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
    // display.init().unwrap();

    // let text_style = MonoTextStyleBuilder::new()
    //     .font(&FONT_6X10)
    //     .text_color(BinaryColor::On)
    //     .build();

    // Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();

    // Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();

    // display.flush().unwrap();

    // Init wifi
    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p.PIN_24, p.PIN_29, p.DMA_CH0);

    let state = make_static!(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let delay = Duration::from_secs(1);
    loop {
        log::info!("led on!");
        control.gpio_set(0, true).await;
        Timer::after(delay).await;

        log::info!("led off!");
        control.gpio_set(0, false).await;
        Timer::after(delay).await;
    }
}
