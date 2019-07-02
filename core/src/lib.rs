use serde::{Deserialize, Serialize};

/* Core Type Definitions */

pub type Vector3 = cgmath::Vector3<f64>;
pub type Vector2 = cgmath::Vector2<f64>;

pub mod primitives;
pub mod scene;

pub use primitives::{Hit, Ray, Vertex};
pub use scene::{Scene, SceneObject};

pub mod prelude {
	pub use super::{
		Hit,
		Intersect,
		Material,
		Ray,
		SurfaceProperties,
		Traceable,
		Vector2,
		Vector3,
		Vertex,
	};

	// We need to ex-export some of cgmath's traits to make certain operations
	// usable without having to leak cgmath
	pub use cgmath::{ElementWise, InnerSpace, MetricSpace};
}

use prelude::*;

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Material {
	Diffuse(Vector3, f64),
	Metal(Vector3, f64),
	Emission(Vector3, Vector3, f64, f64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurfaceProperties {
	pub normal: Vector3,
	pub material: Material,
}

pub trait Intersect {
	fn intersects(&self, ray: Ray) -> Option<Hit>;
}

pub trait Traceable: Intersect {
	fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties;
}
