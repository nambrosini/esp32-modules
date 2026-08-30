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
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

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
    // generator version: 1.3.0
    // generator parameters: --chip esp32 -o esp32-wroom-32 -o unstable-hal

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut trig = Output::new(peripherals.GPIO19, Level::Low, OutputConfig::default());
    let mut echo = Input::new(peripherals.GPIO18, InputConfig::default());
    let delay = Delay::new();

    loop {
        match read_distance_cm(&mut trig, &mut echo, &delay) {
            Some(dist) => {
                esp_println::println!("Distance: {dist} cm")
            }
            None => {
                esp_println::println!("Couldn't read value.")
            }
        }

        delay.delay_millis(500);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}

fn pulse_in(pin: &mut Input, level: bool, timeout: Duration) -> Option<Duration> {
    let start_wait = Instant::now();

    while pin.is_high() == level {
        if Instant::now() - start_wait > timeout {
            return None;
        }
    }

    while pin.is_high() != level {
        if Instant::now() - start_wait > timeout {
            return None;
        }
    }

    let pulse_start = Instant::now();

    while pin.is_high() == level {
        if Instant::now() - start_wait > timeout {
            return None;
        }
    }

    Some(Instant::now() - pulse_start)
}

fn read_distance_cm(trig: &mut Output, echo: &mut Input, delay: &Delay) -> Option<f32> {
    trig.set_low();
    delay.delay_micros(2);
    trig.set_high();
    delay.delay_micros(10);
    trig.set_low();

    let duration = pulse_in(echo, true, Duration::from_millis(30))?;

    let micros = duration.as_micros() as f32;
    Some(micros / 58.0)
}
