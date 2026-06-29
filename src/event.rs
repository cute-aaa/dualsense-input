use crate::types::{Button, DPad, StickSide, TriggerSide};

/// A single touch contact on the touchpad.
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    /// Finger slot (0 or 1).
    pub finger: u8,
    /// X coordinate  (0 – 1919).
    pub x: u16,
    /// Y coordinate  (0 – 1087).
    pub y: u16,
    /// Monotonically-increasing contact id (wraps).
    pub contact_id: u8,
}

/// Every event the library can emit.
#[derive(Debug, Clone)]
pub enum Event {
    /// A button was pressed.
    ButtonPress(Button),
    /// A button was released.
    ButtonRelease(Button),
    /// The D-pad hat direction changed.
    DPadChanged(DPad),
    /// Analog trigger moved.
    Trigger {
        which: TriggerSide,
        value: u8,
    },
    /// Analog stick moved (raw 0-255, center = 128).
    StickMove {
        which: StickSide,
        x: u8,
        y: u8,
    },
    /// Finger touched the pad.
    TouchDown(TouchPoint),
    /// Finger moved on the pad.
    TouchMove(TouchPoint),
    /// Finger left the pad.
    TouchUp { finger: u8 },
    /// Gyroscope reading (pitch, yaw, roll).
    Gyroscope { pitch: i16, yaw: i16, roll: i16 },
    /// Accelerometer reading (x, y, z).
    Accelerometer { x: i16, y: i16, z: i16 },
}
