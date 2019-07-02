use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct AABB {
	pub min: Vector3,
	pub max: Vector3,
}

impl Intersect for AABB {
	fn intersects(&self, ray: Ray) -> Option<Hit> {
		let inverse_ray_dir = 1.0 / ray.direction;
		let mut t1 = (self.min[0] - ray.origin[0]) * inverse_ray_dir[0];
		let mut t2 = (self.max[0] - ray.origin[0]) * inverse_ray_dir[0];

		let mut tmin = t1.min(t2);
		let mut tmax = t1.max(t2);

		for i in 1..3 {
			t1 = (self.min[i] - ray.origin[i]) * inverse_ray_dir[i];
			t2 = (self.max[i] - ray.origin[i]) * inverse_ray_dir[i];

			tmin = tmin.max(t1.min(t2));
			tmax = tmax.min(t1.max(t2));
		}

		if !(tmax > tmin.max(0.0)) {
			return None;
		}

		return Some(Hit::new(ray, tmin));
	}
}
