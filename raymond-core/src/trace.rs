use crate::{geometry::*, math::prelude::*, prelude::*, scene::Scene, brdf::cook_torrance::DefaultCookTorrance};
use cgmath::{InnerSpace, SquareMatrix};
use rand;
use crate::brdf::cook_torrance::CookTorrance;

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
	let normal = surface_properties.normal.normalize();
	let fragment_position = ray.origin + ray.direction * hit.distance;
	let (material_color, material_roughness, material_metalness, emission) = match object.material {
		Material::Diffuse(color, roughness) => (color, roughness, 0.0, Vector3::new(0.0, 0.0, 0.0)),
		Material::Metal(color, roughness) => (color, roughness, 1.0, Vector3::new(0.0, 0.0, 0.0)),
		Material::Emission(e, d, roughness, metalness) => {
			(d, roughness, metalness, e)
		}
	};

	let view_dir = (ray.origin - fragment_position).normalize();

	let f0 = Vector3::new(0.04, 0.04, 0.04);
	let f0 = lerp_vec(f0, material_color, material_metalness);
	// Decide whether to sample diffuse or specular
	let r = rand::random::<TFloat>();
	let local_cartesian = create_coordinate_system_of_n(normal);
	let local_cartesian_transform = cgmath::Matrix3::from_cols(local_cartesian.0, normal, local_cartesian.1);
	let prob_d = 0.1;

	let outgoing_radiance = if r < prob_d {
		let (sample, pdf) = Ray::random_direction_over_hemisphere();
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
		let fresnel = DefaultCookTorrance::fresnel(halfway.dot(view_dir).max(0.0), f0);
		let specular_part = fresnel;
		let diffuse_part = Vector3::new(1.0, 1.0, 1.0) - specular_part;
		let diffuse_part = diffuse_part * (1.0 - material_metalness);
		let output = (diffuse_part.mul_element_wise(material_color)).mul_element_wise(radiance) * cos_theta;
		output / pdf
	} else {
		let world_to_local = local_cartesian_transform.invert().unwrap();
		let (sample_normal_space, pdf) = DefaultCookTorrance::importance_sample(world_to_local * view_dir, material_roughness);
		let sample_world = (local_cartesian_transform * sample_normal_space).normalize();

		if sample_world.dot(normal) < 0.0 {
			return Vector3::new(0.0, 0.0, 0.0);
		}

		let radiance = trace(
			Ray {
				origin: fragment_position + sample_world * 0.000001,
				direction: sample_world,
			},
			scene,
			depth + 1,
			depth_limit,
		);

		let cos_theta = normal.dot(sample_world).max(0.0);
		let light_dir = sample_world.normalize();
		let halfway = (light_dir + view_dir).normalize();
		let F = DefaultCookTorrance::fresnel(halfway.dot(view_dir).max(0.0), f0);
		let D = DefaultCookTorrance::microfacet_distribution(normal.dot(halfway), material_roughness);
		let G = DefaultCookTorrance::geometric_attenuation(view_dir, light_dir, normal, material_roughness);
		let nominator = D * G * F;
		let denominator = 4.0 * normal.dot(view_dir) * cos_theta + 0.001;
		let specular = nominator / denominator;

		let diffuse_part = (Vector3::new(1.0, 1.0, 1.0) - F).mul_element_wise(material_color.mul_element_wise(radiance));

		let output = diffuse_part + (specular.mul_element_wise(radiance));

		output * cos_theta / pdf
	};

	return Vector3::new(
		if outgoing_radiance.x.is_infinite() || outgoing_radiance.x.is_nan() { 0.0 } else { outgoing_radiance.x },
		if outgoing_radiance.y.is_infinite() || outgoing_radiance.y.is_nan() { 0.0 } else { outgoing_radiance.y },
		if outgoing_radiance.z.is_infinite() || outgoing_radiance.z.is_nan() { 0.0 } else { outgoing_radiance.z },
	) + emission
}


fn lerp_vec(min: Vector3, max: Vector3, a: TFloat) -> Vector3 {
	Vector3::new(lerp(min.x, max.x, a), lerp(min.y, max.y, a), lerp(min.z, max.z, a))
}

fn lerp(min: TFloat, max: TFloat, a: TFloat) -> TFloat {
	min + a * (max - min)
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
