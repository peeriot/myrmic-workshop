#![no_std]
#![no_main]

use esp_firmware::{embassy_time, Board, Network};
use esp_firmware::embassy_executor::Spawner;
use esp_firmware::esp_hal::peripherals::Peripherals;

#[esp_firmware::main]
async fn setup(
    board: Board,
    net: Network,
    peripherals: Peripherals,
    spawner: Spawner,
) {
    loop {
        esp_firmware::log::info!("hello");
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
    }
}

