pub mod acc_grid;
pub mod mesh;
pub mod primitives;

use crate::math::prelude::*;

#[derive(Clone, Debug)]
pub struct SurfaceProperties {
	pub normal: Vector3,
}

#[derive(Clone, Copy, Debug)]
pub struct Hit {
	pub distance: TFloat,
	pub ray: Ray,
	pub subobject_index: usize,
}

impl Hit {
	pub fn new(ray: Ray, distance: TFloat) -> Hit {
		Hit {
			ray,
			distance,
			subobject_index: 0,
		}
	}

	pub fn with_child(ray: Ray, distance: TFloat, subobject_index: usize) -> Hit {
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

	pub fn random_direction(origin: Vector3) -> Ray {
		let r1 = rand::random::<TFloat>();
		let r2 = rand::random::<TFloat>();

		let theta = (2.0 * r1 - 1.0).acos() - (PI / 2.0);
		let phi = 2.0 * PI * r2;

		let cartesian = Vector3::new(theta.cos() * phi.cos(), theta.cos() * phi.sin(), theta.sin());

		Ray {
			origin,
			direction: cartesian,
		}
	}

	pub fn random_direction_over_hemisphere() -> (Vector3, TFloat) {
		let r1 = rand::random::<TFloat>();
		let r2 = rand::random::<TFloat>();
	
		let theta = (r1.sqrt()).acos();
		let phi = 2.0 * PI * r2;
	
		let pdf = theta.cos() * (1.0 / PI);
		let cartesian = Vector3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
		return (cartesian, pdf);
	}
}



pub trait Intersect {
	fn intersects(&self, ray: Ray) -> Option<Hit>;
}

pub trait Traceable: Intersect {
	fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties;
}

pub use self::{
	acc_grid::AccGrid,
	mesh::Mesh,
	primitives::{Plane, Sphere, Triangle, Vertex, AABB},
};
