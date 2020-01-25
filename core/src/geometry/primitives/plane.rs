use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Plane {
	pub origin: Vector3,
	pub normal: Vector3,
}

impl Intersect for Plane {
	fn intersects(&self, ray: Ray) -> Option<Hit> {
		let normal = self.normal;

		let denom = normal.dot(-ray.direction);
		if denom > 1e-6 {
			let p0l0 = self.origin - ray.origin;
			let t = p0l0.dot(-normal) / denom;
			if t >= 0.0 {
				return Some(Hit::new(ray, t));
			}
		}

		return None;
	}
}

impl Plane {
	pub fn get_surface_properties(&self, _hit: Hit) -> SurfaceProperties {
		SurfaceProperties { normal: self.normal }
	}
}
