use std::{
	f32,
	sync::{
		atomic::{AtomicUsize, Ordering},
		mpsc::{self},
		Arc,
	},
	thread,
	time::Duration,
};
 use cgmath::InnerSpace;
use crossbeam::queue::ArrayQueue;
use rand;
use num_cpus;
use crate::{geometry::*, prelude::*, scene::Scene, tile::Tile};
use crate::math::prelude::*;
use log::*;



pub fn trace(ray: Ray, scene: &Scene, depth: usize, depth_limit: usize) -> Vector3 {
	if depth > depth_limit {
		return Vector3::new(0.0, 0.0, 0.0);
	}

	let intersect = scene.intersect(ray);
	let (object, hit) = match intersect {
		Some((o, h)) => (o, h),
		None => return Vector3::new(1.0, 1.0, 1.0),
	};
	let surface_properties = object.geometry.get_surface_properties(hit);
	let normal = surface_properties.normal;
	let fragment_position = ray.origin + ray.direction * hit.distance;
	let (material_color, material_roughness, material_metalness) = match object.material {
		Material::Diffuse(color, roughness) => (color, roughness, 0.0),
		Material::Metal(color, roughness) => (color, roughness, 1.0),
		Material::Emission(e, _, _, _) => {
			return e;
		}
		_ => panic!(),
	};

    let view_dir = (ray.origin - fragment_position).normalize();

	let f0 = Vector3::new(0.04, 0.04, 0.04);
	let f0 = lerp_vec(f0, material_color, material_metalness);
	// Decide whether to sample diffuse or specular
	let r = rand::random::<f32>();
	let local_cartesian = create_coordinate_system_of_n(normal);
	let local_cartesian_transform = cgmath::Matrix3::from_cols(local_cartesian.0, normal, local_cartesian.1);
	let prob_d = lerp(0.5, 0.0, material_metalness);
	if r < prob_d {
		let (sample, pdf) = uniform_sample_hemisphere();
		let sample_world = (local_cartesian_transform * sample).normalize();
		let radiance = trace(
			Ray {
				origin: fragment_position + normal * 0.00001,
				direction: sample_world,
			},
			scene,
			depth + 1,
            depth_limit,
		);
		let cos_theta = normal.dot(sample_world).max(0.0);
		let halfway = (sample_world + view_dir).normalize();
		let fresnel = fresnel_schlick(halfway.dot(view_dir).max(0.0), f0);
		let specular_part = fresnel;
		let mut diffuse_part = Vector3::new(1.0, 1.0, 1.0) - specular_part;
		diffuse_part *= 1.0 - material_metalness;
		let output = (diffuse_part.mul_element_wise(material_color)).mul_element_wise(radiance) * cos_theta;
		return prob_d * output / pdf;
	} else {
		// Sample specular
		let reflect = (-view_dir - 2.0 * (-view_dir.dot(normal) * normal)).normalize();
		fn importance_sample_ggx(reflect: Vector3, roughness: f32) -> Vector3 {
			let r1: f32 = rand::random();
			let r2: f32 = rand::random();
			let a = roughness * roughness;
			let phi = 2.0 * PI * r1;
			let theta = a * (r2 / (1.0 - r2)).sqrt();
			let h = Vector3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
			let (tangent, bitangent) = create_coordinate_system_of_n(reflect);
			let matrix = cgmath::Matrix3::from_cols(tangent, reflect, bitangent);
			return (matrix * h).normalize();
		}
		let sample_world = importance_sample_ggx(reflect, material_roughness);
		let radiance = trace(
			Ray {
				origin: fragment_position + normal * 0.0001,
				direction: sample_world,
			},
			scene,
			depth + 1,
            depth_limit,
		);
		let cos_theta = normal.dot(sample_world);
		let light_dir = sample_world.normalize();
		let halfway = (light_dir + view_dir).normalize();
		let F = fresnel_schlick(halfway.dot(view_dir), f0);
		let D = ggx_distribution(normal, halfway, material_roughness);
		let G = geometry_smith(normal, view_dir, sample_world, material_roughness);
		let nominator = D * G * F;
		let denominator = 4.0 * normal.dot(view_dir) * cos_theta + 0.001;
		let specular = nominator / denominator;
		let output = (specular).mul_element_wise(radiance) * cos_theta;
		// pdf
		let pdf = { (D * normal.dot(halfway)) / (4.0 * halfway.dot(view_dir)) + 0.0001 };
		return (1.0 - prob_d) * output / pdf;
	}
}

fn ggx_distribution(n: Vector3, h: Vector3, roughness: f32) -> f32 {
	let a2 = roughness * roughness;
	let NdotH = n.dot(h);

	let nominator = a2;
	let denominator = NdotH.powf(2.0) * (a2 - 1.0) + 1.0;
	let denominator = (PI * denominator * denominator).max(1e-7);
	return nominator / denominator;
}

fn geometry_schlick_ggx(n: Vector3, v: Vector3, r: f32) -> f32 {
	let numerator = n.dot(v).max(0.0);
	let k = (r * r) / 8.0;
	let denominator = numerator * (1.0 - k) + k;

	return numerator / denominator;
}

fn geometry_smith(n: Vector3, v: Vector3, l: Vector3, r: f32) -> f32 {
	return geometry_schlick_ggx(n, v, r) * geometry_schlick_ggx(n, l, r);
}

fn fresnel_schlick(cos_theta: f32, F0: Vector3) -> Vector3 {
	return F0 + (Vector3::new(1.0, 1.0, 1.0) - F0) * (1.0 - cos_theta).powf(5.0);
}

fn lerp_vec(min: Vector3, max: Vector3, a: f32) -> Vector3 {
	Vector3::new(lerp(min.x, max.x, a), lerp(min.y, max.y, a), lerp(min.z, max.z, a))
}

fn lerp(min: f32, max: f32, a: f32) -> f32 {
	min + a * (max - min)
}

fn uniform_sample_hemisphere() -> (Vector3, f32) {
	let r1 = rand::random::<f32>();
	let r2 = rand::random::<f32>();

	let theta = (r1.sqrt()).acos();
	let phi = 2.0 * PI * r2;

	let pdf = r1.sqrt();
	let cartesian = Vector3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
	return (cartesian, pdf);
}

fn create_coordinate_system_of_n(n: Vector3) -> (Vector3, Vector3) {
	let sign = if n.z > 0.0 { 1.0 } else { -1.0 };
	let a = -1.0 / (sign + n.z);
	let b = n.x * n.y * a;
	return (
		Vector3::new(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x),
		Vector3::new(b, sign + n.y * n.y * a, -n.y),
	);
}
