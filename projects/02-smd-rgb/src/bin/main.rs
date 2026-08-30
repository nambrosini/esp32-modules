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
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer};
use esp_hal::main;
use esp_hal::time::Rate;
use log::error;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
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
    // generator parameters: --chip esp32 -o unstable-hal -o esp32-wroom-32 -o log

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let delay = Delay::new();

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(5),
        })
        .unwrap();

    let mut ch_r = ledc.channel(channel::Number::Channel0, peripherals.GPIO25);
    let mut ch_g = ledc.channel(channel::Number::Channel1, peripherals.GPIO26);
    let mut ch_b = ledc.channel(channel::Number::Channel2, peripherals.GPIO27);

    for ch in [&mut ch_r, &mut ch_g, &mut ch_b] {
        ch.configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: esp_hal::gpio::DriveMode::PushPull,
        })
        .unwrap();
    }

    // Fase 0: rosso pieno, verde e blu spenti
    ch_r.set_duty(100).unwrap();
    ch_g.set_duty(0).unwrap();
    ch_b.set_duty(0).unwrap();

    loop {
        fade(&mut ch_r, &mut ch_g, &delay);
        fade(&mut ch_g, &mut ch_b, &delay);
        fade(&mut ch_b, &mut ch_r, &delay);
    }
}

fn fade(
    from: &mut channel::Channel<'_, LowSpeed>,
    to: &mut channel::Channel<'_, LowSpeed>,
    delay: &Delay,
) {
    for pct in 0..=100u8 {
        from.set_duty(100 - pct).unwrap();
        to.set_duty(pct).unwrap();
        delay.delay_millis(15);
    }
}
