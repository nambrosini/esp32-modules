# Ultrasonic Module

## Description

As the ultrasonic has strong directivity, slow energy consumption and far spread distance in the media, so it is commonly used in the measurement of distance, such as range finder and position measuring instrument. Ultrasonic detector module can provide 2cm-450cm non-contact sensing distance, and its ranging accuracy is up to 3mm, very good to meet the normal requirements. The module includes an ultrasonic transmitter and receiver as well as the corresponding control circuit.

## Specification

- Working voltage：0.5V(DC)
- Working current：15mA
- Detecting range：2-450cm
- Detecting angle：15 degrees
- Input trigger pulse：10us TTL Level
- Output echo signal： output TTL level signal(HIGH)，proportional to range.

## Connect

![Image](../images/45-ultrasonic-module.jpg)

## Code

```rust
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
```

## References

- [Hosyond](https://45-in-1-sensor-kit.readthedocs.io/en/latest/45.ultrasonic_module.html)
