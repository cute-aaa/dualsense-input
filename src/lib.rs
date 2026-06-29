//! # dualsense-input
//!
//! Read PS5 **DualSense** controller input over USB / Bluetooth HID.
//!
//! ## Quick start
//!
//! ```no_run
//! use dualsense_input::{DualSense, Event};
//!
//! let mut ds = DualSense::connect()?;
//! ds.on_event(|ev| {
//!     if let Event::ButtonPress(b) = ev {
//!         println!("pressed: {b:?}");
//!     }
//! });
//! ds.listen()?;
//! # Ok::<(), dualsense_input::Error>(())
//! ```

mod types;
mod config;
mod event;
mod controller;

pub use types::*;
pub use config::{Config, ConfigBuilder};
pub use event::{Event, TouchPoint};
pub use controller::{DualSense, Error};
