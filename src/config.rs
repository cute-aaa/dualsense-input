use std::collections::HashSet;
use crate::types::{Button, TriggerSide, StickSide};

/// Runtime configuration — thresholds, dead-zones, and exclusion filters.
///
/// Build one via [`Config::builder()`].
///
/// ```
/// use dualsense_input::{Config, Button};
///
/// let cfg = Config::builder()
///     .stick_deadzone(40)
///     .gyro_threshold(200)
///     .exclude_buttons(&[Button::Mute, Button::PS])
///     .build();
/// ```
pub struct Config {
    pub(crate) stick_deadzone: u8,
    pub(crate) stick_threshold: u8,
    pub(crate) gyro_threshold: u16,
    pub(crate) accel_threshold: u16,
    pub(crate) trigger_threshold: u8,
    pub(crate) excluded_buttons: HashSet<Button>,
    pub(crate) excluded_triggers: HashSet<TriggerSide>,
    pub(crate) excluded_sticks: HashSet<StickSide>,
    pub(crate) report_touch: bool,
    pub(crate) report_gyro: bool,
    pub(crate) report_accel: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stick_deadzone: 30,
            stick_threshold: 5,
            gyro_threshold: 120,
            accel_threshold: 250,
            trigger_threshold: 5,
            excluded_buttons: HashSet::new(),
            excluded_triggers: HashSet::new(),
            excluded_sticks: HashSet::new(),
            report_touch: true,
            report_gyro: true,
            report_accel: true,
        }
    }
}

impl Config {
    /// Start building a configuration.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Returns `true` if this button is **not** excluded.
    pub fn accepts_button(&self, b: &Button) -> bool {
        !self.excluded_buttons.contains(b)
    }
}

// ── builder ───────────────────────────────────────────────────────

#[derive(Default)]
pub struct ConfigBuilder {
    c: Config,
}

impl ConfigBuilder {
    /// Stick distance from 128 to be considered "at rest".  Default **30**, max 127.
    pub fn stick_deadzone(mut self, v: u8) -> Self {
        self.c.stick_deadzone = v.min(127);
        self
    }
    /// Minimum stick position change (per axis) to emit an event.  Default **5**.
    pub fn stick_threshold(mut self, v: u8) -> Self {
        self.c.stick_threshold = v;
        self
    }
    /// Per-axis gyro change that must be exceeded to emit.  Default **120**.
    pub fn gyro_threshold(mut self, v: u16) -> Self {
        self.c.gyro_threshold = v;
        self
    }
    /// Per-axis accel change threshold.  Default **250**.
    pub fn accel_threshold(mut self, v: u16) -> Self {
        self.c.accel_threshold = v;
        self
    }
    /// Analog trigger change threshold.  Default **5**.
    pub fn trigger_threshold(mut self, v: u8) -> Self {
        self.c.trigger_threshold = v;
        self
    }
    /// Exclude a single button from reporting.
    pub fn exclude_button(mut self, b: Button) -> Self {
        self.c.excluded_buttons.insert(b);
        self
    }
    /// Exclude several buttons at once.
    pub fn exclude_buttons(mut self, btns: &[Button]) -> Self {
        for &b in btns {
            self.c.excluded_buttons.insert(b);
        }
        self
    }
    /// Exclude an analog trigger from reporting.
    pub fn exclude_trigger(mut self, t: TriggerSide) -> Self {
        self.c.excluded_triggers.insert(t);
        self
    }
    /// Exclude a stick from reporting.
    pub fn exclude_stick(mut self, s: StickSide) -> Self {
        self.c.excluded_sticks.insert(s);
        self
    }
    /// Toggle touchpad events.  Default **true**.
    pub fn report_touch(mut self, v: bool) -> Self {
        self.c.report_touch = v;
        self
    }
    /// Toggle gyroscope events.  Default **true**.
    pub fn report_gyro(mut self, v: bool) -> Self {
        self.c.report_gyro = v;
        self
    }
    /// Toggle accelerometer events.  Default **true**.
    pub fn report_accel(mut self, v: bool) -> Self {
        self.c.report_accel = v;
        self
    }
    /// Produce the final [`Config`].
    pub fn build(self) -> Config {
        self.c
    }
}
