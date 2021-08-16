/* Core Type Definitions */

use serde::{Deserialize, Serialize};

pub type Vector3 = cgmath::Vector3<f64>;
pub type Vector2 = cgmath::Vector2<f64>;

pub mod geometry;
pub mod math;
pub mod project;
pub mod scene;
pub mod tile;

pub use tile::Tile;

pub mod prelude {
	pub use super::{
		geometry::{Hit, Intersect, Ray, SurfaceProperties, Traceable, Vertex},
		math::prelude::*,
		Material,
	};
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Material {
	Diffuse(Vector3, f64),
	Metal(Vector3, f64),
	Emission(Vector3, Vector3, f64, f64),
}

use bbox_intersect::{Array_i32_1d, Error, FutharkContext};

pub fn do_magic() -> Result<(), Error> {
	let a = vec![1, 2, 3, 4];
	let b = vec![2, 3, 4, 1];

	let mut ctx = FutharkContext::new()?;
	let a_arr = Array_i32_1d::from_vec(ctx, &a, &vec![4])?;
	let b_arr = Array_i32_1d::from_vec(ctx, &b, &vec![4])?;

	let res = ctx.bbox_intersect(a_arr, b_arr)?;

	println!("{:?}", res);
	return Ok(())
}