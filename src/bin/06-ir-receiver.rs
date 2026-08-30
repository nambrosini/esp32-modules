#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::delay::Delay;
use esp_hal::main;
use esp_hal::rmt::{PulseCode, Rmt, RxChannelConfig, RxChannelCreator};
use esp_hal::time::Rate;
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

    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    let rx_config = RxChannelConfig::default()
        .with_clk_divider(80) // 1 tick = 1 microsecondo
        .with_idle_threshold(12000); // fine frame dopo 12ms di silenzio

    let mut channel = rmt
        .channel2
        .configure_rx(&rx_config)
        .unwrap()
        .with_pin(peripherals.GPIO5);

    let mut buffer = [PulseCode::default(); 68];

    println!("In ascolto sul GPIO5... punta il telecomando Samsung e premi un tasto");

    loop {
        channel = match channel.receive(&mut buffer) {
            Ok(transaction) => match transaction.wait() {
                Ok((_len, channel)) => {
                    if let Some((address, command)) = decode_nec_samsung(&buffer) {
                        println!(
                            "Comando ricevuto -> address: 0x{:04X}, command: 0x{:02X}",
                            address, command
                        );
                    } else {
                        println!(
                            "Frame ricevuto ma non riconosciuto (rumore o protocollo diverso)"
                        );
                    }
                    channel
                }
                Err((_e, channel)) => channel, // timeout: nessun tasto premuto, normale
            },
            Err((_e, channel)) => channel,
        };
        delay.delay_millis(50);
    }
}

/// Decodifica un frame NEC/Samsung a 32 bit da un buffer di PulseCode
/// catturato dall'RMT. Ritorna (address, command) se il frame è valido.
fn decode_nec_samsung(buffer: &[PulseCode]) -> Option<(u16, u8)> {
    // Il primo elemento è il preambolo (es. ~4500us high + ~4500us low per
    // Samsung, o ~9000/4500 per NEC classico) - lo saltiamo e partiamo dal
    // secondo elemento per leggere i 32 bit dati.
    if buffer.len() < 33 {
        return None;
    }

    let mut frame: u32 = 0;
    for i in 0..32 {
        let pulse = buffer[i + 1];
        if pulse.length1() == 0 {
            return None; // frame troppo corto / interrotto
        }
        // Un bit '1' ha un basso lungo (~1690us), un bit '0' un basso corto
        // (~560us). Soglia a 1000us per distinguere i due casi.
        let bit: u32 = if pulse.length2() > 1000 { 1 } else { 0 };
        frame = (frame << 1) | bit;
    }

    let address = (frame >> 16) as u16;
    let command = ((frame >> 8) & 0xFF) as u8;
    let inverted_command = (frame & 0xFF) as u8;

    // Verifica checksum: il secondo byte comando deve essere il
    // complemento del primo (come nel protocollo NEC classico).
    if command != !inverted_command {
        return None;
    }

    Some((address, command))
}
