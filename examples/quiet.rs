use dualsense_input::{DualSense, Config, Event};

/// 静默模式示例：调高阈值，只输出有意义的事件（按键、大动作）
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        // 摇杆：更大的死区和变化阈值，忽略微小漂移
        .stick_deadzone(50)
        .stick_threshold(15)
        // 陀螺仪：大幅提高，只在明显转动时输出
        .gyro_threshold(3000)
        // 加速度计：大幅提高，只在明显移动/震动时输出
        .accel_threshold(8000)
        // 扳机：提高阈值
        .trigger_threshold(15)
        // 关闭触摸板事件（大量滑动时会刷屏）
        .report_touch(false)
        // 关闭陀螺仪和加速度计（静置时也在漂移）
        .report_gyro(false)
        .report_accel(false)
        .build();

    let mut ds = DualSense::connect_with_config(config)?;
    println!("DualSense connected (quiet mode) — only buttons, sticks, triggers");

    ds.on_event(|ev| {
        match ev {
            Event::ButtonPress(b)   => println!("PRESS   {b:?}"),
            Event::ButtonRelease(b) => println!("RELEASE {b:?}"),
            Event::DPadChanged(d)   => println!("D-Pad   {d}"),
            Event::Trigger { which, value } => {
                println!("{which:?} = {value}");
            }
            Event::StickMove { which, x, y } => {
                println!("{which:?} stick ({x}, {y})");
            }
            // 以下事件已被 config 关闭，不会触发
            _ => {}
        }
    });

    ds.listen()?;
    Ok(())
}
