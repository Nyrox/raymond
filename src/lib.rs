extern crate num_traits;
extern crate cgmath;
extern crate image;
extern crate rand;
extern crate crossbeam_utils;
extern crate num_cpus;

pub mod mesh;
pub mod primitives;
pub mod scene;
pub mod trace;
pub mod acc_grid;
pub mod material;

pub type F = f64;
pub const F_MAX: F = ::std::f64::MAX;
pub const PI: F = ::std::f64::consts::PI;

pub use cgmath::Vector3;

pub struct Tile {
	pub width: usize,
	pub height: usize,
	pub left: usize,
	pub top: usize,
	pub data: Vec<cgmath::Vector3<F>>,
}


pub mod prelude {
	pub use cgmath::Vector3;
	pub use mesh::Mesh;
	pub use primitives::{Plane};
	pub use scene::{Scene, Object, Model};
	pub use acc_grid::AccGrid;
	pub use material::Material;
	pub use trace::RaytracerConfig;

}