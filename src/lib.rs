

#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate derive_builder;

pub mod acc_grid;
pub mod material;
pub mod mesh;
pub mod primitives;
pub mod scene;
pub mod trace;
pub mod transform;

pub type F = f64;
pub const F_MAX: F = ::std::f64::MAX;
pub const PI: F = ::std::f64::consts::PI;

extern crate cgmath;
pub use cgmath::Vector3;

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
	pub sample_count: usize,
	pub width: usize,
	pub height: usize,
	pub left: usize,
	pub top: usize,
	pub data: Vec<cgmath::Vector3<F>>,
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
		material::Material,
		mesh::Mesh,
		primitives::Plane,
		scene::{Model, Object, Scene},
	};
}
