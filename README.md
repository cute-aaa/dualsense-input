# dualsense-input

Read PS5 **DualSense** controller input — buttons, sticks, triggers, touchpad,
gyroscope, and accelerometer — from Rust, with configurable thresholds and
button exclusion.

| Feature | Status |
|---|---|
| All face / shoulder / system buttons | ✅ |
| D-pad (hat switch) | ✅ |
| Analog triggers (L2 / R2) | ✅ |
| Analog sticks with dead-zone | ✅ |
| Touchpad (2-finger, coordinates + contact id) | ✅ |
| Gyroscope (pitch / yaw / roll) | ✅ |
| Accelerometer (x / y / z) | ✅ |
| USB & Bluetooth | ✅ |
| Exclusion filters (buttons, sticks, triggers) | ✅ |
| Configurable per-channel thresholds | ✅ |

## Add to your project

```toml
[dependencies]
dualsense-input = "0.1"
```

## Quick start — callback (blocking)

```rust
use dualsense_input::{DualSense, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ds = DualSense::connect()?;
    ds.on_event(|ev| {
        if let Event::ButtonPress(b) = ev {
            println!("{b:?}");
        }
    });
    ds.listen()?;
    Ok(())
}
```

## Quick start — polling (non-blocking)

```rust,no_run
use dualsense_input::{DualSense, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ds = DualSense::connect()?;
    loop {
        for ev in ds.poll()? {
            match ev {
                Event::Trigger { which, value } => println!("{which:?} = {value}"),
                _ => {}
            }
        }
        // your game loop here …
    }
}
```

## Custom configuration

```rust
use dualsense_input::{Config, DualSense, Button};

let config = Config::builder()
    .stick_deadzone(40)
    .gyro_threshold(200)
    .accel_threshold(300)
    .exclude_buttons(&[Button::Mute, Button::PS])
    .report_touch(false)
    .build();

let ds = DualSense::connect_with_config(config)?;
```

## License

MIT
