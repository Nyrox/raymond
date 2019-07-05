use crate::Transform;

use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::prelude::*, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
	pub backbuffer_width: usize,
	pub backbuffer_height: usize,
	pub fov_vert: f64,
	pub transform: Transform,
	pub focal_length: f64,
	pub aperture_radius: f64,
}

impl Default for CameraSettings {
	fn default() -> Self {
		CameraSettings {
			backbuffer_width: 1280,
			backbuffer_height: 720,
			fov_vert: 75.0,
			transform: Transform::default(),
			focal_length: 2.5,
			aperture_radius: 0.0,
		}
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WorkerCount {
	Auto,
	Fixed(usize),
}

impl WorkerCount {
	pub fn get(self) -> usize {
		match self {
			Self::Auto => num_cpus::get(),
			Self::Fixed(c) => c,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSettings {
	pub worker_count: WorkerCount,
	pub samples_per_pixel: usize,
	pub bounce_limit: usize,
}

impl Default for TraceSettings {
	fn default() -> Self {
		TraceSettings {
			worker_count: WorkerCount::Auto,
			samples_per_pixel: 30,
			bounce_limit: 5,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
	pub camera: CameraSettings,
	pub trace: TraceSettings,
}

impl Settings {
	pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
		let f = File::open(path.as_ref())?;
		Ok(ron::de::from_reader(f)?)
	}
}
