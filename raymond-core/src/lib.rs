/* Core Type Definitions */

use prelude::TFloat;
use serde::{Deserialize, Serialize};

pub type Vector3 = cgmath::Vector3<TFloat>;
pub type Vector2 = cgmath::Vector2<TFloat>;

pub mod geometry;
pub mod math;
pub mod project;
pub mod scene;
pub mod tile;
pub mod trace;
pub mod brdf;

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
	Diffuse(Vector3, TFloat),
	Metal(Vector3, TFloat),
	Emission(Vector3, Vector3, TFloat, TFloat),
}
