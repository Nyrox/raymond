use crate::prelude::*;

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
