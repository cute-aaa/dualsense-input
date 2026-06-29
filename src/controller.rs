use hidapi::{HidApi, HidDevice, HidError};

use crate::config::Config;
use crate::event::{Event, TouchPoint};
use crate::types::*;

// ── HID identifiers ───────────────────────────────────────────────
const SONY_VID: u16 = 0x054C;
const DUALSENSE_PID: u16 = 0x0CE6;

// ── report byte offsets (after removing Report ID / BT header) ────
// Based on verified working offsets from Linux hid-playstation.c
const O_LX: usize = 0;
const O_LY: usize = 1;
const O_RX: usize = 2;
const O_RY: usize = 3;
const O_L2: usize = 4;
const O_R2: usize = 5;
const O_BTN0: usize = 7;
const O_BTN1: usize = 8;
const O_BTN2: usize = 9;
const O_GYRO_PITCH: usize = 15; // int16 LE
const O_GYRO_YAW: usize = 17;
const O_GYRO_ROLL: usize = 19;
const O_ACCEL_X: usize = 21; // int16 LE
const O_ACCEL_Y: usize = 23;
const O_ACCEL_Z: usize = 25;
const O_TOUCH0: usize = 32; // 4 bytes
const O_TOUCH1: usize = 36; // 4 bytes

const DATA_MIN_LEN: usize = 40;

const TOUCHPAD_MAX_X: u16 = 1920;
const TOUCHPAD_MAX_Y: u16 = 1088;

// ── previous-state tracker ────────────────────────────────────────
#[derive(Default, Clone)]
struct PrevState {
    buttons: u32,
    dpad: u8,
    lx: u8,
    ly: u8,
    rx: u8,
    ry: u8,
    l2: u8,
    r2: u8,
    gyro: [i16; 3],
    accel: [i16; 3],
    gyro_init: bool,
    accel_init: bool,
    touch0_active: bool,
    touch0_id: u8,
    touch1_active: bool,
    touch1_id: u8,
}

// ── public controller handle ──────────────────────────────────────

/// A connected DualSense controller.
pub struct DualSense {
    device: HidDevice,
    config: Config,
    prev: PrevState,
    callback: Option<Box<dyn FnMut(&Event)>>,
}

impl DualSense {
    /// Open the first DualSense found (USB or BT).
    pub fn connect() -> Result<Self, Error> {
        Self::connect_with_config(Config::default())
    }

    /// Open with a custom [`Config`].
    pub fn connect_with_config(config: Config) -> Result<Self, Error> {
        let api = HidApi::new()?;
        let mut best = None;

        for info in api.device_list() {
            if info.vendor_id() == SONY_VID && info.product_id() == DUALSENSE_PID {
                best = Some(info);
                // Prefer Gamepad usage (page=0x01, usage=0x05)
                if info.usage_page() == 0x01 && info.usage() == 0x05 {
                    break;
                }
            }
        }

        let di = best.ok_or(Error::NotFound)?;
        let device = api.open_path(di.path())?;

        Ok(Self {
            device,
            config,
            prev: PrevState::default(),
            callback: None,
        })
    }

    /// Register a callback for every emitted event.
    pub fn on_event<F: FnMut(&Event) + 'static>(&mut self, f: F) {
        self.callback = Some(Box::new(f));
    }

    /// **Blocking** event loop — reads reports forever and fires callbacks.
    pub fn listen(&mut self) -> Result<(), Error> {
        let mut buf = [0u8; 128];
        loop {
            let n = self.device.read_timeout(&mut buf, 10)? as usize;
            if n < 2 {
                continue;
            }
            if let Some(data) = Self::extract_report(&buf, n) {
                if data.len() >= DATA_MIN_LEN {
                    let events = self.process_report(data);
                    for ev in &events {
                        if let Some(cb) = &mut self.callback {
                            cb(ev);
                        }
                    }
                }
            }
        }
    }

    /// **Non-blocking** poll — reads all currently-available reports
    /// and returns every generated event.
    pub fn poll(&mut self) -> Result<Vec<Event>, Error> {
        let mut all = Vec::new();
        let mut buf = [0u8; 128];
        loop {
            match self.device.read_timeout(&mut buf, 0) {
                Ok(n) if n >= 2 => {
                    if let Some(data) = Self::extract_report(&buf, n) {
                        if data.len() >= DATA_MIN_LEN {
                            all.extend(self.process_report(data));
                        }
                    }
                }
                Ok(_) => break,
                Err(HidError::HidApiError { .. }) => break,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(all)
    }

    /// Returns a reference to the active [`Config`].
    pub fn config(&self) -> &Config {
        &self.config
    }

    // ── report extraction ─────────────────────────────────────────

    /// Extract the data portion from raw HID buffer, handling various
    /// report formats (USB/BT, with/without Report ID).
    fn extract_report<'a>(buf: &'a [u8], n: usize) -> Option<&'a [u8]> {
        match n {
            63 => Some(&buf[..63]),                           // USB, no Report ID
            64 if buf[0] == 0x01 => Some(&buf[1..64]),       // USB, with Report ID
            64 => Some(&buf[..64]),                           // USB, non-standard
            77 => Some(&buf[2..]),                            // BT, no Report ID
            78 if buf[0] == 0x01 => Some(&buf[3..]),         // BT, with Report ID
            78 => Some(&buf[2..]),                            // BT, non-standard
            _ => None,
        }
    }

    // ── report processing ─────────────────────────────────────────

    fn process_report(&mut self, report: &[u8]) -> Vec<Event> {
        let mut events = Vec::new();

        // ── buttons ───────────────────────────────────────────────
        let btn0 = report[O_BTN0];
        let btn1 = report[O_BTN1];
        let btn2 = report[O_BTN2];
        let new_btns = pack_buttons(btn0, btn1, btn2);
        let old_btns = self.prev.buttons;
        if new_btns != old_btns {
            let changed = new_btns ^ old_btns;
            let mut bit = 1u32;
            for _ in 0..24 {
                if changed & bit != 0 {
                    if let Some(btn) = bit_to_button(bit) {
                        if self.config.accepts_button(&btn) {
                            if new_btns & bit != 0 {
                                events.push(Event::ButtonPress(btn));
                            } else {
                                events.push(Event::ButtonRelease(btn));
                            }
                        }
                    }
                }
                bit <<= 1;
            }
            self.prev.buttons = new_btns;
        }

        // ── d-pad ─────────────────────────────────────────────────
        let dpad_raw = btn0 & 0x0F;
        if dpad_raw != self.prev.dpad && self.config.accepts_button(&Button::DPadUp) {
            events.push(Event::DPadChanged(DPad::from_nibble(dpad_raw)));
            self.prev.dpad = dpad_raw;
        }

        // ── triggers (analog) ─────────────────────────────────────
        let trigger_threshold = self.config.trigger_threshold;
        let excl_left = self.config.excluded_triggers.contains(&TriggerSide::Left);
        let excl_right = self.config.excluded_triggers.contains(&TriggerSide::Right);

        let l2 = report[O_L2];
        if !excl_left && (l2 as i16 - self.prev.l2 as i16).unsigned_abs() as u8 > trigger_threshold {
            events.push(Event::Trigger { which: TriggerSide::Left, value: l2 });
            self.prev.l2 = l2;
        }
        let r2 = report[O_R2];
        if !excl_right && (r2 as i16 - self.prev.r2 as i16).unsigned_abs() as u8 > trigger_threshold {
            events.push(Event::Trigger { which: TriggerSide::Right, value: r2 });
            self.prev.r2 = r2;
        }

        // ── sticks ────────────────────────────────────────────────
        let lx = report[O_LX];
        let ly = report[O_LY];
        self.emit_stick(StickSide::Left, lx, ly, &mut events);
        let rx = report[O_RX];
        let ry = report[O_RY];
        self.emit_stick(StickSide::Right, rx, ry, &mut events);

        // ── touchpad ──────────────────────────────────────────────
        let report_touch = self.config.report_touch;
        let report_accel = self.config.report_accel;
        let report_gyro = self.config.report_gyro;
        let accel_threshold = self.config.accel_threshold;
        let gyro_threshold = self.config.gyro_threshold;

        if report_touch && report.len() >= O_TOUCH1 + 4 {
            self.process_touch(report, &mut events);
        }

        // ── accelerometer ─────────────────────────────────────────
        if report_accel && report.len() >= O_ACCEL_Z + 2 {
            let ax = i16::from_le_bytes([report[O_ACCEL_X], report[O_ACCEL_X + 1]]);
            let ay = i16::from_le_bytes([report[O_ACCEL_Y], report[O_ACCEL_Y + 1]]);
            let az = i16::from_le_bytes([report[O_ACCEL_Z], report[O_ACCEL_Z + 1]]);
            if self.prev.accel_init {
                let dx = (ax - self.prev.accel[0]).unsigned_abs();
                let dy = (ay - self.prev.accel[1]).unsigned_abs();
                let dz = (az - self.prev.accel[2]).unsigned_abs();
                if dx > accel_threshold || dy > accel_threshold || dz > accel_threshold {
                    events.push(Event::Accelerometer { x: ax, y: ay, z: az });
                }
            }
            self.prev.accel = [ax, ay, az];
            self.prev.accel_init = true;
        }

        // ── gyroscope ─────────────────────────────────────────────
        if report_gyro && report.len() >= O_GYRO_ROLL + 2 {
            let gp = i16::from_le_bytes([report[O_GYRO_PITCH], report[O_GYRO_PITCH + 1]]);
            let gy = i16::from_le_bytes([report[O_GYRO_YAW], report[O_GYRO_YAW + 1]]);
            let gr = i16::from_le_bytes([report[O_GYRO_ROLL], report[O_GYRO_ROLL + 1]]);
            if self.prev.gyro_init {
                let dp = (gp - self.prev.gyro[0]).unsigned_abs();
                let dy = (gy - self.prev.gyro[1]).unsigned_abs();
                let dr = (gr - self.prev.gyro[2]).unsigned_abs();
                if dp > gyro_threshold || dy > gyro_threshold || dr > gyro_threshold {
                    events.push(Event::Gyroscope { pitch: gp, yaw: gy, roll: gr });
                }
            }
            self.prev.gyro = [gp, gy, gr];
            self.prev.gyro_init = true;
        }

        events
    }

    // ── stick helper ──────────────────────────────────────────────

    fn emit_stick(&mut self, which: StickSide, x: u8, y: u8, out: &mut Vec<Event>) {
        if self.config.excluded_sticks.contains(&which) {
            return;
        }
        let (px, py) = match which {
            StickSide::Left => (self.prev.lx, self.prev.ly),
            StickSide::Right => (self.prev.rx, self.prev.ry),
        };
        let dz = self.config.stick_deadzone;
        let thr = self.config.stick_threshold;

        let dx = (x as i16 - px as i16).unsigned_abs() as u8;
        let dy = (y as i16 - py as i16).unsigned_abs() as u8;
        if dx <= thr && dy <= thr {
            return;
        }

        let within_deadzone =
            |val: u8| (val as i16 - 128).unsigned_abs() as u8 <= dz;

        if within_deadzone(x) && within_deadzone(y) && within_deadzone(px) && within_deadzone(py) {
            return;
        }

        out.push(Event::StickMove { which, x, y });
        match which {
            StickSide::Left => { self.prev.lx = x; self.prev.ly = y; }
            StickSide::Right => { self.prev.rx = x; self.prev.ry = y; }
        }
    }

    // ── touchpad helper ───────────────────────────────────────────

    /// Parse touch data from the report.
    /// Touch point format (4 bytes):
    ///   byte[0]: bit7 = inactive (1=lifted), bit0-6 = contact ID
    ///   X = byte[1] | ((byte[2] & 0x0F) << 8)   → 12-bit
    ///   Y = (byte[2] >> 4) | (byte[3] << 4)      → 12-bit
    fn process_touch(&mut self, report: &[u8], out: &mut Vec<Event>) {
        for i in 0u8..2 {
            let base = O_TOUCH0 + (i as usize) * 4;
            if base + 4 > report.len() { break; }
            let b = &report[base..base + 4];

            let active = (b[0] & 0x80) == 0;
            let contact_id = b[0] & 0x7F;
            let x = (b[1] as u16) | (((b[2] & 0x0F) as u16) << 8);
            let y = ((b[2] >> 4) as u16) | ((b[3] as u16) << 4);

            // discard out-of-range noise that appears when idle
            let in_range = x < TOUCHPAD_MAX_X && y < TOUCHPAD_MAX_Y;

            let (was_active, prev_id) = if i == 0 {
                (self.prev.touch0_active, self.prev.touch0_id)
            } else {
                (self.prev.touch1_active, self.prev.touch1_id)
            };

            let tp = TouchPoint { finger: i, x, y, contact_id };

            match (was_active && in_range, active && in_range) {
                (false, true) => out.push(Event::TouchDown(tp)),
                (true, true) if contact_id == prev_id => out.push(Event::TouchMove(tp)),
                (true, true) => {
                    out.push(Event::TouchUp { finger: i });
                    out.push(Event::TouchDown(tp));
                }
                (true, false) => out.push(Event::TouchUp { finger: i }),
                _ => {}
            }

            if i == 0 {
                self.prev.touch0_active = active && in_range;
                self.prev.touch0_id = contact_id;
            } else {
                self.prev.touch1_active = active && in_range;
                self.prev.touch1_id = contact_id;
            }
        }
    }
}

// ── error type ────────────────────────────────────────────────────

/// Errors produced by the library.
#[derive(Debug)]
pub enum Error {
    /// No DualSense controller was found on any HID bus.
    NotFound,
    /// Low-level HID error.
    Hid(hidapi::HidError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "DualSense controller not found"),
            Self::Hid(e) => write!(f, "HID error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Hid(e) => Some(e),
            _ => None,
        }
    }
}

impl From<hidapi::HidError> for Error {
    fn from(e: hidapi::HidError) -> Self {
        Self::Hid(e)
    }
}
