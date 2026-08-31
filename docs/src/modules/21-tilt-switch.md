# 21 Tilt Switch

## Description

Tilt Sensor is a digital tilt switch. It can be used as a simple tilt switch.
Simply plug it to our IO/Sensor shield, easy for wire connection. With dedicated
sensor shield and Arduino, you can make lots of interesting and interactive works.

## Specification

- Supply Voltage: 3.3V to 5V
- Interface: Digital
- Size: 30*20mm
- Weight: 3g

## Connect

![Image](../images/21-tilt-switch.jpg)

## Code

```rust
#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let switch = Input::new(peripherals.GPIO25, InputConfig::default());

    loop {
        led.set_level(switch.level());
        delay.delay_millis(5);
    }
}
```

## References

- [Hosyond 45 in 1 Sensor Kit Documentation](https://45-in-1-sensor-kit.readthedocs.io/en/latest/21.tilt_switch.html)
