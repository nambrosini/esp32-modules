# Cheatsheet

Riferimento per la nuova API stabile di `esp-hal` (rilasciata ottobre 2025). Con la 1.0 sono stabili (garanzie SemVer) solo: inizializzazione (`esp_hal::init`), GPIO, UART, SPI, I2C (sync + async), il modulo `time`, poche API di sistema e la macro `#[main]`. **Tutto il resto — inclusi ADC, PWM, RMT — sta dietro alla feature `unstable`**, senza garanzie di stabilità tra versioni minori.

## Cargo.toml

```toml
[dependencies]
esp-hal = { version = "1.0", features = ["esp32c3", "unstable"] }  # "unstable" serve per ADC/PWM/ecc.
esp-backtrace = { version = "0.14", features = ["esp32c3", "panic-handler", "println"] }
esp-println = { version = "0.12", features = ["esp32c3"] }
```

---

## 1. Inizializzazione delle peripherals

Non esiste più `Peripherals::take()` + `SystemControl` + `ClockControl`. Un'unica funzione fa tutto e restituisce direttamente i singleton delle periferiche.

```rust
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::main;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    loop {}
}
```

Per configurare il clock (es. massima frequenza):

```rust
use esp_hal::clock::CpuClock;

let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
let peripherals = esp_hal::init(config);
```

Note:

- L'entry point è `#[main]` (da `esp_hal::main`), non più `#[entry]`.
- `peripherals` contiene i campi per ogni periferica (es. `peripherals.GPIO8`, `peripherals.I2C0`, `peripherals.ADC1`).
- Se serve riusare una periferica in più punti (es. driver temporaneo), si usa `.reborrow()` invece di spostarla.

---

## 2. Digitale: leggere e scrivere GPIO

### Output (scrittura)

```rust
use esp_hal::gpio::{Level, Output, OutputConfig};

let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

led.set_high();
led.set_low();
led.toggle();
```

### Input (lettura)

```rust
use esp_hal::gpio::{Input, InputConfig, Pull};

let config = InputConfig::default().with_pull(Pull::Up); // pull-up interno, es. per un pulsante
let button = Input::new(peripherals.GPIO0, config);

if button.is_low() {
    // pulsante premuto (assumendo pull-up e pulsante verso GND)
}
```

Note:

- Non serve più passare per `Io::new(...).pins.gpioX` — il pin si prende direttamente da `peripherals.GPIOx`.
- `OutputConfig`/`InputConfig` sostituiscono i vecchi parametri sciolti nel costruttore; usa `::default()` se non ti servono opzioni particolari (drive strength, pull, ecc.).

---

## 3. ADC: leggere un ingresso analogico

⚠️ L'ADC è dietro la feature `unstable` — API non coperta da garanzie SemVer, può cambiare tra versioni minori.

```rust
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    main,
};
use esp_println::println;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut adc_config = AdcConfig::new();
    let mut adc_pin = adc_config.enable_pin(peripherals.GPIO2, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    loop {
        let valore: u16 = adc.read_oneshot(&mut adc_pin).unwrap();
        println!("ADC: {}", valore);
    }
}
```

Note:

- `Attenuation` determina il range di tensione leggibile (più attenuazione = range più ampio, meno precisione a tensioni basse). `_11dB` copre circa 0–3.1V, buono come default per sensori alimentati a 3.3V.
- Il valore letto è raw (tipicamente 12 bit → 0–4095), non già convertito in volt: la conversione a tensione va fatta a mano in base alla risoluzione e all'attenuazione scelta.
- L'ESP32 "classico" ha ADC1 (GPIO32-39) e ADC2 (condiviso con WiFi, da evitare se usi WiFi); le varianti C3/C6/S3 hanno mappature pin diverse — controlla il datasheet della tua board.

---

## 4. LEDC (PWM)

⚠️ Anche LEDC è dietro la feature `unstable`.

LEDC (LED Controller) è la periferica PWM dell'ESP32: genera un'onda quadra su un pin con un duty cycle regolabile (0-100%), utile per intensità LED, motori, servo, o come "DAC via PWM filtrato" (vedi sezione ADC). Serve un **timer** (frequenza + risoluzione) collegato a uno o più **channel** (uno per pin).

```rust
use esp_hal::{
    ledc::{channel, timer, LSGlobalClkSource, Ledc, LowSpeed},
    time::Rate,
};

let mut ledc = Ledc::new(peripherals.LEDC);
ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
lstimer0.configure(timer::config::Config {
    duty: timer::config::Duty::Duty10Bit, // risoluzione: 1024 livelli
    clock_source: timer::LSClockSource::APBClk,
    frequency: Rate::from_khz(5),
}).unwrap();

let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO8);
channel0.configure(channel::config::Config {
    timer: &lstimer0,
    duty_pct: 50, // 0-100
    drive_mode: channel::config::DriveMode::PushPull,
}).unwrap();

// Per cambiare intensità/duty in seguito:
channel0.set_duty(80).unwrap();

// Fade automatico via hardware (senza intervento della CPU):
channel0.start_duty_fade(0, 100, 1000).unwrap(); // da 0% a 100% in 1000ms
```

Note:

- Su ESP32 classico ci sono 8 canali, divisi in gruppo `LowSpeed` e `HighSpeed`; `LowSpeed` è l'unico disponibile su tutte le varianti (C3/C6 non hanno `HighSpeed`), quindi conviene usarlo come default.
- Frequenza e risoluzione del duty sono legate: frequenza più alta → risoluzione massima disponibile più bassa (limite hardware del timer).
- Più canali possono condividere lo stesso timer (es. per pilotare i 3 pin R/G/B di un LED RGB con un unico timer e 3 channel separati).

---

## 5. Delay

```rust
use esp_hal::delay::Delay;

let delay = Delay::new(); // non serve più passare i &clocks

delay.delay_millis(500);
delay.delay_micros(100);
```

Per attese più lunghe o per non bloccare la CPU (utile se in futuro passi ad Embassy/async), valuta i timer (`esp_hal::timer`) invece del delay bloccante.

---

## Riepilogo cambi principali rispetto alle versioni precedenti alla 1.0

| Prima | Ora (1.0+) |
| --- | --- |
| `Peripherals::take()` + `SystemControl` + `ClockControl::...freeze()` | `esp_hal::init(Config::default())` |
| `#[entry]` | `#[main]` |
| `Io::new(...).pins.gpioX` | `peripherals.GPIOx` diretto |
| `Delay::new(&clocks)` | `Delay::new()` |
| `Spi::new_with_config(...)` | `Spi::new(peripherals, Config::default())?` |
| Quasi tutto disponibile di default | Solo GPIO/UART/SPI/I2C stabili; resto (ADC, LEDC/PWM, ecc.) dietro `unstable` |
