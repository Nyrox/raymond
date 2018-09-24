extern crate cgmath;
extern crate crossbeam;
extern crate crossbeam_utils;
extern crate image;
extern crate num_cpus;
extern crate num_traits;
extern crate rand;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod acc_grid;
pub mod material;
pub mod mesh;
pub mod primitives;
pub mod scene;
pub mod trace;

pub type F = f64;
pub const F_MAX: F = ::std::f64::MAX;
pub const PI: F = ::std::f64::consts::PI;

pub use cgmath::Vector3;

#[derive(Clone, Serialize)]
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
	pub use acc_grid::AccGrid;
	pub use cgmath::Vector3;
	pub use material::Material;
	pub use mesh::Mesh;
	pub use primitives::Plane;
	pub use scene::{Model, Object, Scene};
	pub use trace::RaytracerConfig;
}
