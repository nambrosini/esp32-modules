#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, Level, Output, OutputConfig};
use esp_hal::main;
use esp_println::println;
use onewire::ds18b20::FAMILY_CODE;
use onewire::{DS18B20, DeviceSearch, OneWire, Sensor};

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
    let mut delay = Delay::new();

    // Pin dati DS18B20 su GPIO4, come open-drain (richiesto dal protocollo 1-Wire)
    // Serve un resistore di pull-up esterno (4.7kΩ) tra dato e VCC
    let mut one_wire_pin = Output::new(
        peripherals.GPIO4,
        Level::High,
        OutputConfig::default().with_drive_mode(DriveMode::OpenDrain),
    )
    .into_flex();
    one_wire_pin.set_input_enable(true);

    let mut one_wire_bus = OneWire::new(one_wire_pin, false);

    loop {
        // Enumera i device sul bus e legge ciascuno
        let mut search = DeviceSearch::new();
        while let Some(device) = one_wire_bus.search_next(&mut search, &mut delay).unwrap() {
            if device.family_code() != FAMILY_CODE {
                continue; // non è un DS18B20
            }

            let sensor = DS18B20::new(device).unwrap();

            // Avvia la conversione e attende il tempo richiesto dalla risoluzione corrente
            let wait_ms = sensor
                .start_measurement(&mut one_wire_bus, &mut delay)
                .unwrap();
            delay.delay_millis(wait_ms as u32);

            let temperature = sensor
                .read_measurement(&mut one_wire_bus, &mut delay)
                .unwrap();

            println!("Temp: {:.2}°C", temperature);
        }

        delay.delay_millis(1000);
    }
}
