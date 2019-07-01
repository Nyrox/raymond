use crate::prelude::*;
use crate::primitives::AABB;

#[derive(Debug)]
pub struct Triangle(pub Vertex, pub Vertex, pub Vertex);

impl Intersect for Triangle {
	fn intersects(&self, ray: Ray) -> Option<Hit> {
		const EPSILON: f64 = 0.00000001;

		let (vertex0, vertex1, vertex2) = (self.0.position, self.1.position, self.2.position);

		let edge1 = vertex1 - vertex0;
		let edge2 = vertex2 - vertex0;

		let h = ray.direction.cross(edge2);
		let a = edge1.dot(h);
		if a < EPSILON && a > -EPSILON {
			return None;
		}

		let f = 1.0 / a;
		let s = ray.origin - vertex0;
		let u = f * s.dot(h);
		if u < 0.0 || u > 1.0 {
			return None;
		}

		let q = s.cross(edge1);
		let v = f * ray.direction.dot(q);
		if v < 0.0 || u + v > 1.0 {
			return None;
		}

		let t = f * edge2.dot(q);
		if t > EPSILON {
			return Some(Hit::new(ray, t));
		}

		return None;
	}


}
impl Triangle {
	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		fn area(a: Vector3, b: Vector3, c: Vector3) -> f64 {
			let ab = a.distance(b);
			let ac = a.distance(c);
			let bc = b.distance(c);
			let s = (ab + ac + bc) / 2.0;
			return (s * (s - ab) * (s - ac) * (s - bc)).sqrt();
		}

		let position = hit.ray.origin + hit.ray.direction * hit.distance;
		let abc = area(self.0.position, self.1.position, self.2.position);
		let abp = area(self.0.position, self.1.position, position);
		let bcp = area(self.0.position, self.2.position, position);
		let ba = abp / abc;
		let bb = bcp / abc;
		let bc = 1.0 - (ba + bb);
		let normal = (self.2.normal * ba) + (self.1.normal * bb) + (self.0.normal * bc);

		return SurfaceProperties {
			normal: normal.normalize(),
			material: Material::Diffuse(Vector3::new(0.0, 0.0, 0.0), 0.5)
		};
	}

	pub fn find_bounds(&self) -> AABB {
		let mut min = Vector3::new(125125.0, 1251251.0, 12512512.0);
		let mut max = Vector3::new(-123125.0, -125123.0, -512123.0);

		for i in 0..3 {
			min[i] = min[i].min(self.0.position[i]);
			min[i] = min[i].min(self.1.position[i]);
			min[i] = min[i].min(self.2.position[i]);
			max[i] = max[i].max(self.0.position[i]);
			max[i] = max[i].max(self.1.position[i]);
			max[i] = max[i].max(self.2.position[i]);
		}

		return AABB { min, max };
	}
}