#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate derive_builder;

pub mod acc_grid;
pub mod mesh;
pub mod scene;
pub mod trace;
pub mod transform;

pub const F_MAX: f64 = ::std::f64::MAX;
pub const PI: f64 = ::std::f64::consts::PI;

extern crate cgmath;

use core::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
	pub sample_count: usize,
	pub width: usize,
	pub height: usize,
	pub left: usize,
	pub top: usize,
	pub data: Vec<Vector3>,
}

use std::fmt;

impl fmt::Debug for Tile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"Tile {{ sample_count: {}, width: {}, height: {}, left: {}, top: {} }}",
			self.sample_count, self.width, self.height, self.left, self.top
		)
	}
}

pub mod prelude {
	pub use crate::{
		acc_grid::AccGrid,
		cgmath::Vector3,
		mesh::Mesh,
		scene::{Model, Scene},
	};

	pub use core::SceneObject;
}
