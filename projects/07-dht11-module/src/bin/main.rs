#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp32_dht11_rs::DHT11;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let mut dht11 = DHT11::new(peripherals.GPIO15, delay);

    loop {
        match dht11.read() {
            Ok(m) => {
                // qui puoi usare esp_println::println! per stampare su seriale
                let temp = m.temperature;
                let hum = m.humidity;
                esp_println::println!("Temp: {temp}, Hum: {hum}");
            }
            Err(_e) => {
                esp_println::println!("Cannot read");
            }
        }
        delay.delay_millis(2000); // il DHT11 non va letto più spesso di 1-2 volte al secondo
    }
}
