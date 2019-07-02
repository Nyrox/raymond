use crate::prelude::*;

#[derive(Clone)]
pub struct Plane {
	pub origin: Vector3,
	pub normal: Vector3,
	pub material: Material,
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
	pub fn get_material(&self) -> Material {
		self.material.clone()
	}

	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		SurfaceProperties {
			normal: self.normal,
			material: self.material,
		}
	}
}
