#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level};
use esp_hal::main;
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const MIN: i32 = 0;
const MAX: i32 = 100;

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let clk = Input::new(peripherals.GPIO15, InputConfig::default());
    let dt = Input::new(peripherals.GPIO18, InputConfig::default());
    let _sw = Input::new(peripherals.GPIO19, InputConfig::default());

    let mut position: i32 = 0;
    let mut last_clk = clk.level();

    loop {
        let current_clk = clk.level();
        if current_clk != last_clk && current_clk == Level::Low {
            if dt.level() != current_clk {
                position += if position < MAX { 1 } else { 0 };
            } else {
                position -= if position > MIN { 1 } else { 0 };
            }
            println!("Position: {position}");
        }
        last_clk = current_clk;

        delay.delay_millis(1);
    }
}
