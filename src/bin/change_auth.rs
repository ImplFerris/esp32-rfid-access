#![no_std]
#![no_main]

use embassy_executor::Spawner;
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

use esp32_rfid_access::{self as lib};

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

    // reset to 0xFF, if you want
    // const DATA: [u8; 16] = [
    //     0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Key A: "Rusted"
    //     0xFF, 0x07, 0x80, 0x69, // Access bits and trailer byte
    //     0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Key B: "Ferris"
    // ];
    // let current_key = &[0x52, 0x75, 0x73, 0x74, 0x65, 0x64];
    let new_key: &[u8; 6] = &lib::rfid::AUTH_DATA[..6].try_into().unwrap(); // First 6 bytes of the block
    let current_key = &lib::rfid::DEFAULT_KEY;
    const DATA: [u8; 16] = lib::rfid::AUTH_DATA;
    let mut rfid = lib::rfid::Rfid::new(rc522);
    loop {
        if rfid.select_card().await {
            println!("Selected Card");
            println!("\r\n----Before Write----");
            rfid.print_sector(lib::rfid::AUTH_SECTOR, current_key)
                .unwrap();

            rfid.write_block(
                lib::rfid::AUTH_SECTOR,
                lib::rfid::SECTOR_TRAILER,
                DATA,
                current_key,
            )
            .unwrap();

            println!("\r\n----After Write----");
            rfid.print_sector(lib::rfid::AUTH_SECTOR, new_key).unwrap();
            rfid.halt_state().unwrap();
        }
    }
}
