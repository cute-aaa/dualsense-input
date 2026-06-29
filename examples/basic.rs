use dualsense_input::{DualSense, Config, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .stick_deadzone(30)
        .stick_threshold(5)
        .gyro_threshold(120)
        .accel_threshold(250)
        // silence buttons you don't care about:
        // .exclude_buttons(&[Button::Mute, Button::Touchpad])
        .build();

    let mut ds = DualSense::connect_with_config(config)?;
    println!("DualSense connected — listening …");

    ds.on_event(|ev| {
        match ev {
            Event::ButtonPress(b)   => println!("PRESS   {b:?}"),
            Event::ButtonRelease(b) => println!("RELEASE {b:?}"),
            Event::DPadChanged(d)   => println!("D-Pad   {d}"),
            Event::Trigger { which, value }
                                    => println!("{which:?} = {value}"),
            Event::StickMove { which, x, y }
                                    => println!("{which:?} stick ({x}, {y})"),
            Event::TouchDown(tp)    => println!("touch↓ finger={} ({},{})", tp.finger, tp.x, tp.y),
            Event::TouchMove(tp)    => println!("touch→ finger={} ({},{})", tp.finger, tp.x, tp.y),
            Event::TouchUp { finger }=>println!("touch↑ finger={finger}"),
            Event::Gyroscope { pitch, yaw, roll }
                                    => println!("gyro  p={pitch} y={yaw} r={roll}"),
            Event::Accelerometer { x, y, z }
                                    => println!("accel x={x} y={y} z={z}"),
        }
    });

    ds.listen()?;
    Ok(())
}
