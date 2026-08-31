#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Pull};
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

    let mut adc_config = AdcConfig::new();
    let mut vrx_pin = adc_config.enable_pin(peripherals.GPIO34, Attenuation::_11dB);
    let mut vry_pin = adc_config.enable_pin(peripherals.GPIO35, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    let sw = Input::new(
        peripherals.GPIO32,
        InputConfig::default().with_pull(Pull::Up),
    );

    loop {
        let x: u16 = nb::block!(adc.read_oneshot(&mut vrx_pin)).unwrap();
        let y: u16 = nb::block!(adc.read_oneshot(&mut vry_pin)).unwrap();
        let pressed = sw.is_low();

        println!("X: {x:<4} | Y: {y:<4} | Pressed: {pressed}");

        delay.delay_millis(100);
    }
}
