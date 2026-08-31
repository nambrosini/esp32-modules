#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::analog::adc::{Adc, AdcConfig};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_println::println;

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

    let mut adc_config = AdcConfig::default();
    let mut adc_pin =
        adc_config.enable_pin(peripherals.GPIO32, esp_hal::analog::adc::Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    let mut power = Output::new(peripherals.GPIO14, Level::Low, OutputConfig::default());

    loop {
        power.set_high();

        delay.delay_millis(10);

        let raw_value = nb::block!(adc.read_oneshot(&mut adc_pin)).unwrap();
        let water_level = (raw_value as f32) / 4095.0 * 100.0;
        println!("Raw: {raw_value}, Level: {:.2}%", water_level);

        power.set_low();

        delay.delay_millis(1000);
    }
}
