use std::{fs, path::PathBuf};

use super::{F_MAX, PI};

use core::{
	prelude::*,
	primitives::{Triangle, AABB},
};

#[derive(Debug)]
pub struct Mesh {
	pub triangles: Vec<Triangle>,
	pub bounding_box: AABB,
}

impl Mesh {
	pub fn new(triangles: Vec<Triangle>) -> Self {
		Mesh {
			bounding_box: Self::find_mesh_bounds(&triangles),
			triangles,
		}
	}

	pub fn intersects(&self, ray: Ray) -> Option<Hit> {
		self.bounding_box.intersects(ray)?;
		let mut closest = F_MAX;
		let mut closest_hit = None;

		for (i, tri) in self.triangles.iter().enumerate() {
			match tri.intersects(ray) {
				Some(h) => {
					let distance = h.distance;
					if distance < closest {
						closest = distance;
						closest_hit = Some(Hit::with_child(ray, closest, i));
					}
				}
				None => {}
			}
		}

		closest_hit
	}

	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		self.triangles[hit.subobject_index].get_surface_properties(hit)
	}

	pub fn bake_transform(&mut self, translate: Vector3) {
		for mut triangle in self.triangles.iter_mut() {
			triangle.0.position += translate;
			triangle.1.position += translate;
			triangle.2.position += translate;
		}

		self.bounding_box = Self::find_mesh_bounds(&self.triangles);
	}

	pub fn load_ply(path: PathBuf) -> Mesh {
		let buffer = fs::read_to_string(path).unwrap();

		let mut vertices = Vec::<Vertex>::new();
		let mut faces = Vec::<Triangle>::new();
		let mut lines = buffer.lines();

		// Parse header
		'header: while let Some(line) = lines.next() {
			let mut tokens = line.split_whitespace();

			match tokens.next().unwrap() {
				"element" => match tokens.next().unwrap() {
					"vertex" => {
						vertices.reserve_exact(tokens.next().unwrap().parse::<usize>().unwrap())
					}
					_ => {}
				},
				"end_header" => break 'header,
				_ => {}
			}
		}

		// Parse vertices
		for _ in 0..vertices.capacity() {
			let mut line = lines.next().unwrap();
			let mut tokens = line.split_whitespace();
			let values = tokens
				.map(|t| t.parse::<f64>().unwrap())
				.collect::<Vec<f64>>();
			vertices.push(Vertex {
				position: Vector3::new(values[0], values[1], values[2]),
				normal: Vector3::new(values[3], values[4], values[5]),
				uv: Vector2::new(
					*values.get(6).unwrap_or(&0.0),
					*values.get(7).unwrap_or(&0.0),
				),
				tangent: Vector3::new(0.0, 0.0, 0.0),
			});
		}

		// Parse faces
		'faces: while let Some(line) = lines.next() {
			let mut tokens = line.split_whitespace();
			let values = tokens
				.map(|t| t.parse::<u32>().unwrap())
				.collect::<Vec<u32>>();

			match values[0] {
				3 => {
					let mut face = [values[1], values[2], values[3]];

					let tangent = Vertex::calculate_tangent(
						vertices[face[0] as usize].clone(),
						vertices[face[1] as usize].clone(),
						vertices[face[2] as usize].clone(),
					);
					vertices[face[0] as usize].tangent = tangent;
					vertices[face[1] as usize].tangent = tangent;
					vertices[face[2] as usize].tangent = tangent;

					faces.push(Triangle(
						vertices[values[1] as usize].clone(),
						vertices[values[2] as usize].clone(),
						vertices[values[3] as usize].clone(),
					));
				}
				_ => {}
			}
		}

		Mesh::new(faces)
	}

	pub fn find_mesh_bounds(tris: &Vec<Triangle>) -> AABB {
		let mut min = Vector3::new(125125.0, 1251251.0, 12512512.0);
		let mut max = Vector3::new(-123125.0, -125123.0, -512123.0);

		for tri in tris {
			for i in 0..3 {
				min[i] = min[i].min(tri.0.position[i]);
				min[i] = min[i].min(tri.1.position[i]);
				min[i] = min[i].min(tri.2.position[i]);

				max[i] = max[i].max(tri.0.position[i]);
				max[i] = max[i].max(tri.1.position[i]);
				max[i] = max[i].max(tri.2.position[i]);
			}
		}

		return AABB { min, max };
	}
}
