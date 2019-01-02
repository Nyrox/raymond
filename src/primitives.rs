use cgmath::{prelude::*, *};

use super::F;

use super::material::Material;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
	pub origin: Vector3<F>,
	pub direction: Vector3<F>,
}

impl Ray {
	pub fn new(origin: Vector3<F>, direction: Vector3<F>) -> Ray {
		Ray { origin, direction }
	}
}

#[derive(Clone, Debug)]
pub struct SurfaceProperties {
	pub normal: Vector3<F>,
}

#[derive(Clone, Copy, Debug)]
pub struct Hit {
	pub distance: F,
	pub ray: Ray,
	pub subobject_index: usize,
}

impl Hit {
	pub fn new(ray: Ray, distance: F) -> Hit {
		Hit {
			ray,
			distance,
			subobject_index: 0,
		}
	}

	pub fn with_child(ray: Ray, distance: F, subobject_index: usize) -> Hit {
		Hit {
			ray,
			distance,
			subobject_index,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Vertex {
	pub position: Vector3<F>,
	pub normal: Vector3<F>,
	pub uv: Vector2<F>,
	pub tangent: Vector3<F>,
}

impl Vertex {
	pub fn calculate_tangent(x: Vertex, y: Vertex, z: Vertex) -> Vector3<F> {
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

#[derive(Clone, Debug)]
pub struct AABB {
	pub min: Vector3<F>,
	pub max: Vector3<F>,
}

use std::mem;

impl AABB {
	pub fn intersects(&self, ray: Ray) -> Option<Hit> {
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

#[derive(Debug)]
pub struct Triangle(pub Vertex, pub Vertex, pub Vertex);

impl Triangle {
	pub fn intersects(&self, ray: Ray) -> Option<Hit> {
		const EPSILON: F = 0.00000001;

		let (vertex0, vertex1, vertex2) = (self.0.position, self.1.position, self.2.position);

		let edge1 = vertex1 - vertex0;
		let edge2 = vertex2 - vertex0;

		let h = ray.direction.cross(edge2);
		let a = edge1.dot(h);
		if (a < EPSILON && a > -EPSILON) {
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

	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		fn area(a: Vector3<F>, b: Vector3<F>, c: Vector3<F>) -> F {
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
		};
	}

	pub fn find_bounds(&self) -> AABB {
		let mut min = Vector3::<F>::new(125125.0, 1251251.0, 12512512.0);
		let mut max = Vector3::<F>::new(-123125.0, -125123.0, -512123.0);

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

#[derive(Clone)]
pub struct Plane {
	pub origin: Vector3<F>,
	pub normal: Vector3<F>,
	pub material: Material,
}

impl Plane {
	pub fn get_material(&self) -> Material {
		self.material.clone()
	}

	pub fn intersects(&self, ray: Ray) -> Option<Hit> {
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

	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		SurfaceProperties { normal: self.normal }
	}
}
