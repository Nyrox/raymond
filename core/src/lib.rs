/* Core Type Definitions */

use serde::{Serialize, Deserialize};

pub type Vector3 = cgmath::Vector3<f64>;
pub type Vector2 = cgmath::Vector2<f64>;

pub mod geometry;
pub mod project;
pub mod scene;
pub mod tile;
pub mod math;

pub use tile::Tile;

pub mod prelude {
	pub use super::{
		geometry::{
			Hit, Intersect, Ray, SurfaceProperties, Traceable, Vertex
	 }, Material, math::prelude::*};
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Material {
	Diffuse(Vector3, f64),
	Metal(Vector3, f64),
	Emission(Vector3, Vector3, f64, f64),
}





