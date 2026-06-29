use std::fmt;

/// Physical buttons on the DualSense controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    // Face buttons
    Cross,
    Circle,
    Square,
    Triangle,
    // D-pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    // Shoulder
    L1,
    R1,
    // Trigger digital click (full press)
    L2Digital,
    R2Digital,
    // Stick clicks
    L3,
    R3,
    // System
    Create,
    Options,
    PS,
    Touchpad,
    Mute,
}

/// Analog trigger side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriggerSide {
    Left,
    Right,
}

/// Analog stick side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StickSide {
    Left,
    Right,
}

/// Decoded D-pad hat direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DPad {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
    Released,
}

impl DPad {
    /// Decode the 4-bit hat value from BTN0 lower nibble.
    pub(crate) fn from_nibble(v: u8) -> Self {
        match v & 0x0F {
            0 => Self::Up,
            1 => Self::UpRight,
            2 => Self::Right,
            3 => Self::DownRight,
            4 => Self::Down,
            5 => Self::DownLeft,
            6 => Self::Left,
            7 => Self::UpLeft,
            _ => Self::Released,
        }
    }
}

impl fmt::Display for DPad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Released => write!(f, "Released"),
            other => write!(f, "{:?}", other),
        }
    }
}

// ── internal helpers ──────────────────────────────────────────────

/// Decode all currently-held buttons from the three raw button bytes.
#[allow(dead_code)]
pub(crate) fn buttons_from_bytes(btn0: u8, btn1: u8, btn2: u8) -> Vec<Button> {
    let mut v = Vec::with_capacity(12);
    // btn0 upper nibble → face buttons
    if btn0 & (1 << 4) != 0 { v.push(Button::Square); }
    if btn0 & (1 << 5) != 0 { v.push(Button::Cross); }
    if btn0 & (1 << 6) != 0 { v.push(Button::Circle); }
    if btn0 & (1 << 7) != 0 { v.push(Button::Triangle); }
    // btn1
    if btn1 & (1 << 0) != 0 { v.push(Button::L1); }
    if btn1 & (1 << 1) != 0 { v.push(Button::R1); }
    if btn1 & (1 << 2) != 0 { v.push(Button::L2Digital); }
    if btn1 & (1 << 3) != 0 { v.push(Button::R2Digital); }
    if btn1 & (1 << 4) != 0 { v.push(Button::Create); }
    if btn1 & (1 << 5) != 0 { v.push(Button::Options); }
    if btn1 & (1 << 6) != 0 { v.push(Button::L3); }
    if btn1 & (1 << 7) != 0 { v.push(Button::R3); }
    // btn2
    if btn2 & (1 << 0) != 0 { v.push(Button::PS); }
    if btn2 & (1 << 1) != 0 { v.push(Button::Touchpad); }
    if btn2 & (1 << 2) != 0 { v.push(Button::Mute); }
    v
}

/// Pack the three button bytes into a single u32 (lower 24 bits) for
/// easy change-detection via XOR.
pub(crate) fn pack_buttons(btn0: u8, btn1: u8, btn2: u8) -> u32 {
    (btn0 as u32) | ((btn1 as u32) << 8) | ((btn2 as u32) << 16)
}

/// Turn a set bit position in the packed u32 back into a Button.
pub(crate) fn bit_to_button(bit: u32) -> Option<Button> {
    const B_SQUARE: u32   = 1 << 4;
    const B_CROSS: u32    = 1 << 5;
    const B_CIRCLE: u32   = 1 << 6;
    const B_TRIANGLE: u32 = 1 << 7;
    const B_L1: u32       = 1 << 8;
    const B_R1: u32       = 1 << 9;
    const B_L2D: u32      = 1 << 10;
    const B_R2D: u32      = 1 << 11;
    const B_CREATE: u32   = 1 << 12;
    const B_OPTIONS: u32  = 1 << 13;
    const B_L3: u32       = 1 << 14;
    const B_R3: u32       = 1 << 15;
    const B_PS: u32       = 1 << 16;
    const B_TOUCH: u32    = 1 << 17;
    const B_MUTE: u32     = 1 << 18;

    match bit {
        B_SQUARE   => Some(Button::Square),
        B_CROSS    => Some(Button::Cross),
        B_CIRCLE   => Some(Button::Circle),
        B_TRIANGLE => Some(Button::Triangle),
        B_L1       => Some(Button::L1),
        B_R1       => Some(Button::R1),
        B_L2D      => Some(Button::L2Digital),
        B_R2D      => Some(Button::R2Digital),
        B_CREATE   => Some(Button::Create),
        B_OPTIONS  => Some(Button::Options),
        B_L3       => Some(Button::L3),
        B_R3       => Some(Button::R3),
        B_PS       => Some(Button::PS),
        B_TOUCH    => Some(Button::Touchpad),
        B_MUTE     => Some(Button::Mute),
        _ => None,
    }
}
