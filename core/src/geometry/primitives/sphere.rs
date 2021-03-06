use crate::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Sphere {
	pub origin: Vector3,
	pub radius: f64,
}

impl Intersect for Sphere {
	fn intersects(&self, ray: Ray) -> Option<Hit> {
		let c = self.origin - ray.origin;
		let mut t = c.dot(ray.direction);
		let q = c - t * ray.direction;
		let p = q.dot(q);

		if p > self.radius * self.radius {
			return None;
		}

		t -= (self.radius * self.radius - p).sqrt();
		if t <= 0.0 {
			return None;
		}

		return Some(Hit::new(ray, t));
	}
}

impl Sphere {
	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		SurfaceProperties {
			normal: ((hit.ray.origin + hit.ray.direction * hit.distance) - self.origin).normalize(),
		}
	}
}
