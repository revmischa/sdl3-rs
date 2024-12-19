//! # Getting started
//!
//! ```rust,no_run
//! extern crate sdl3;
//!
//! use sdl3::pixels::Color;
//! use sdl3::event::Event;
//! use sdl3::keyboard::Keycode;
//! use std::time::Duration;
//!
//! pub fn main() {
//!     let sdl_context = sdl3::init().unwrap();
//!     let video_subsystem = sdl_context.video().unwrap();
//!
//!     let window = video_subsystem.window("rust-sdl3 demo", 800, 600)
//!         .position_centered()
//!         .build()
//!         .unwrap();
//!
//!     let mut canvas = window.into_canvas();
//!
//!     canvas.set_draw_color(Color::RGB(0, 255, 255));
//!     canvas.clear();
//!     canvas.present();
//!     let mut event_pump = sdl_context.event_pump().unwrap();
//!     let mut i = 0;
//!     'running: loop {
//!         i = (i + 1) % 255;
//!         canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
//!         canvas.clear();
//!         for event in event_pump.poll_iter() {
//!             match event {
//!                 Event::Quit {..} |
//!                 Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
//!                     break 'running
//!                 },
//!                 _ => {}
//!             }
//!         }
//!         // The rest of the game loop goes here...
//!
//!         canvas.present();
//!         ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
//!     }
//! }
//! ```

#![crate_name = "sdl3"]
#![crate_type = "lib"]
#![allow(clippy::cast_lossless, clippy::transmute_ptr_to_ref, clippy::missing_transmute_annotations, clippy::missing_safety_doc)]

#[macro_use]
extern crate bitflags;
#[cfg(feature = "gfx")]
extern crate c_vec;
#[macro_use]
extern crate lazy_static;
pub extern crate libc;
pub extern crate sdl3_sys as sys;
// use sdl3_sys as sys;

pub use crate::sdl::*;

pub mod clipboard;
pub mod cpuinfo;
#[macro_use]
mod macros;
pub mod audio;
pub mod dialog;
pub mod event;
pub mod filesystem;
pub mod gamepad;
pub mod haptic;
pub mod hint;
pub mod iostream;
pub mod joystick;
pub mod keyboard;
pub mod log;
pub mod messagebox;
pub mod mouse;
pub mod pixels;
pub mod properties;
pub mod rect;
pub mod render;
mod sdl;
#[cfg(feature = "hidapi")]
pub mod sensor;
pub mod surface;
pub mod timer;
pub mod touch;
pub mod url;
pub mod version;
pub mod video;

// modules
#[cfg(feature = "gfx")]
pub mod gfx;
#[cfg(feature = "image")]
pub mod image;
#[cfg(feature = "mixer")]
pub mod mixer;
#[cfg(feature = "ttf")]
pub mod ttf;

mod common;
// Export return types and such from the common module.
pub use crate::common::IntegerOrSdlError;

mod guid;
#[cfg(feature = "raw-window-handle")]
pub mod raw_window_handle;
mod util;
