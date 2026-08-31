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
use esp_hal::{Config, main};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const B: f64 = 3950.0;

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(Config::default());

    let mut adc_config = AdcConfig::new();
    let mut adc_pin =
        adc_config.enable_pin(peripherals.GPIO34, esp_hal::analog::adc::Attenuation::_11dB);

    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    let delay = Delay::new();

    loop {
        let raw_value: u16 = nb::block!(adc.read_oneshot(&mut adc_pin)).unwrap();

        let temp_c = calculate_ntc_temperature(raw_value);
        println!("Raw: {:<4} | Temp: {:.1}°C", raw_value, temp_c);

        delay.delay_millis(500);
    }
}

/// Convert Voltage to Celsius using the Beta Parameter Equation
/// Assumes:
/// - 10k Ohm Resistor in series (standard on KY-013)
/// - 3.3V Supply Voltage
/// - Beta Value = 3950 (standard for Hosyond kits)
fn calculate_ntc_temperature(raw_value: u16) -> f64 {
    1. / (libm::log(1. / (4096. / raw_value as f64 - 1.)) / B + 1.0 / 298.15) - 273.15
}
