pub mod primitives;
pub mod mesh;
pub mod acc_grid;

use crate::math::prelude::*;

#[derive(Clone, Debug)]
pub struct SurfaceProperties {
	pub normal: Vector3,
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
	fn intersects(&self, ray: Ray) -> Option<Hit>;
}

pub trait Traceable: Intersect {
	fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties;
}


pub use self::{
	primitives::{AABB, Sphere, Plane, Triangle, Vertex}, mesh::Mesh, acc_grid::AccGrid
};
