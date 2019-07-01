
/* Core Type Definitions */

pub type Vector3 = cgmath::Vector3<f64>;
pub type Vector2 = cgmath::Vector2<f64>;

pub mod primitives;



pub mod prelude {
	pub use super::{
		Vector2,
		Vector3,
		Material,
		SurfaceProperties,
		Hit,
		Ray,
		Vertex,
		Intersect,
		Traceable,
	};

	// We need to ex-export some of cgmath's traits to make certain operations
	// usable without having to leak cgmath
	pub use cgmath::{
		InnerSpace,
		MetricSpace,
		ElementWise,
	};
}

use prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Material {
	Diffuse(Vector3, f64),
	Metal(Vector3, f64),
	Emission(Vector3, Vector3, f64, f64),
}



#[derive(Clone, Debug)]
pub struct SurfaceProperties {
	pub normal: Vector3,
	pub material: Material
}

#[derive(Clone, Copy, Debug)]
pub struct Hit {
	pub distance: f64,
	pub ray: Ray,
	pub subobject_index: usize,
}

impl Hit {
	pub fn new(ray: Ray, distance: f64) -> Hit {
		Hit {
			ray,
			distance,
			subobject_index: 0,
		}
	}

	pub fn with_child(ray: Ray, distance: f64, subobject_index: usize) -> Hit {
		Hit {
			ray,
			distance,
			subobject_index,
		}
	}
}


#[derive(Clone, Copy, Debug)]
pub struct Ray {
	pub origin: Vector3,
	pub direction: Vector3,
}

impl Ray {
	pub fn new(origin: Vector3, direction: Vector3) -> Ray {
		Ray { origin, direction }
	}
}

pub trait Intersect {
	fn intersects (&self, ray: Ray) -> Option<Hit>;
}

pub trait Traceable: Intersect {
	fn get_surface_properties (&self, hit: Hit) -> SurfaceProperties;
}

#[derive(Clone, Debug)]
pub struct Vertex {
	pub position: Vector3,
	pub normal: Vector3,
	pub uv: Vector2,
	pub tangent: Vector3,
}

impl Vertex {
	pub fn calculate_tangent(x: Vertex, y: Vertex, z: Vertex) -> Vector3 {
		let edge1 = y.position - x.position;
		let edge2 = z.position - x.position;

		let uv1 = y.uv - x.uv;
		let uv2 = z.uv - x.uv;

		let f = 1.0 / (uv1.x * uv2.y - uv2.x * uv1.y);
		let mut tangent = Vector3::new(0.0, 0.0, 0.0);
		tangent.x = f * (uv2.y * edge1.x - uv1.y * edge2.x);
		tangent.y = f * (uv2.y * edge1.y - uv1.y * edge2.y);
		tangent.z = f * (uv2.y * edge1.z - uv1.y * edge2.z);

		tangent.normalize()
	}
}