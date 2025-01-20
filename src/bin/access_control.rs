#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output},
    prelude::*,
    spi::{
        master::{Config, Spi},
        SpiMode,
    },
};
use esp_println::println;
use log::info;
use mfrc522::{comm::blocking::spi::SpiInterface, Mfrc522};

use esp32_rfid_access as lib;

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");
    let delay = Delay::new();

    let spi = Spi::new_with_config(
        peripherals.SPI2,
        Config {
            frequency: 5.MHz(),
            mode: SpiMode::Mode0,
            ..Config::default()
        },
    )
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO23)
    .with_miso(peripherals.GPIO19);
    let sd_cs = Output::new(peripherals.GPIO5, Level::High);
    let spi = ExclusiveDevice::new(spi, sd_cs, delay).unwrap();

    let spi_interface = SpiInterface::new(spi);
    let rc522 = Mfrc522::new(spi_interface).init().unwrap();
    let mut rfid = lib::rfid::Rfid::new(rc522);

    #[cfg(feature = "oled")]
    let mut display = {
        let i2c0 = esp_hal::i2c::master::I2c::new(
            peripherals.I2C0,
            esp_hal::i2c::master::Config {
                frequency: 400.kHz(),
                timeout: Some(100),
            },
        )
        .with_scl(peripherals.GPIO32)
        .with_sda(peripherals.GPIO33)
        .into_async();

        lib::display::Display::new(i2c0).await
    };

    #[cfg(feature = "servo")]
    let mut servo = {
        let ledc = esp_hal::ledc::Ledc::new(peripherals.LEDC);
        lib::servo::Motor::new(ledc, peripherals.GPIO27)
    };

    loop {
        #[cfg(feature = "oled")]
        display.wait_for_auth().await;

        if rfid.select_card().await {
            println!("Card Present");
            if rfid.authenticate().is_err() {
                #[cfg(feature = "oled")]
                display.acccess_denied().await;
            } else {
                #[cfg(feature = "oled")]
                display.acccess_granted().await;

                #[cfg(feature = "servo")]
                servo.open_door().await;
            }
            rfid.halt_state().unwrap();
            Timer::after_secs(2).await;
        }

        #[cfg(feature = "servo")]
        servo.close_door().await;
        // Timer::after_millis(50).await;
    }
}
