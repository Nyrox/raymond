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

use crossbeam::queue::ArrayQueue;
use rand;

use num_cpus;

use super::transform::Transform;

use raymond_core::{geometry::*, prelude::*, scene::Scene, tile::Tile};

use super::PI;

use log::*;

#[derive(Builder, Clone, Debug)]
pub struct CameraSettings {
	pub backbuffer_width: usize,
	pub backbuffer_height: usize,
	pub fov_vert: f32,
	pub transform: Transform,
	pub focal_length: f32,
	pub aperture_radius: f32,
}

#[derive(Builder, Clone, Debug)]
pub struct Settings {
	#[builder(default = "num_cpus::get()")]
	pub worker_count: usize,

	pub camera_settings: CameraSettings,
	pub sample_count: usize,
	// The amount of samples per iteration
	// If set to 0, the renderer will only notify the Receiver when a tile is fully rendered
	#[builder(default = "0")]
	pub samples_per_iteration: usize,
	pub tile_size: (usize, usize),
	pub bounce_limit: usize,
}

struct TraceContext {
	pub scene: Scene,
	pub settings: Settings,
}

#[derive(Clone, Debug)]
pub enum Message {
	Finished,
	TileFinished(Tile),
	TileProgressed(Tile),
}

pub type TileCallback = Box<dyn Fn(Tile) -> ()>;


pub struct TaskHandle {
	pub receiver: mpsc::Receiver<Message>,
	pub settings: Settings,
	callback: Option<TileCallback>,
	alive_thread_count: Arc<AtomicUsize>,
}



impl TaskHandle {
	pub fn set_callback(&mut self, callback: Option<TileCallback>) {
		self.callback = callback;
	}

	pub fn r#await(&self) -> Vec<Vector3> {
		let mut out = vec![
			Vector3::new(0.0, 0.0, 0.0);
			self.settings.camera_settings.backbuffer_width * self.settings.camera_settings.backbuffer_height
		];

		'poll: loop {
			if self.alive_thread_count.load(Ordering::Relaxed) == 0 {
				'collect: loop {
					match self.receiver.try_recv() {
						Ok(Message::TileFinished(tile)) => {
							for y in 0..tile.height {
								for x in 0..tile.width {
									let s = tile.data[x + y * tile.width] / tile.sample_count as f32;

									out[x + tile.left + (y + tile.top) * self.settings.camera_settings.backbuffer_width] = s;
								}
							}
						}
						_ => {
							break 'collect;
						}
					}
				}

				break 'poll;
			}
			thread::sleep(Duration::from_millis(500));
		}

		out
	}

	pub fn poll(&self) -> Option<Message> {
		self.receiver.try_recv().ok()
	}

	pub fn async_await(&self) -> () {
		'collect: loop {
			match self.receiver.try_recv() {
				Ok(Message::TileProgressed(tile)) => {
					println!("CB");
					if let Some(cb) = &self.callback {
						cb(tile);
					}
				}
				_ => {
					break 'collect;
				}
			}
			thread::sleep(Duration::from_millis(200));
		}
	}
}

pub fn render_tiled(scene: Scene, settings: Settings) -> TaskHandle {
	let mut initial_tiles = Vec::new();
	let (sender, receiver) = mpsc::channel();

	// Split the backbuffer into tiles and push them into the queue
	{
		let mut x = 0;
		let mut y = 0;

		'gen_tiles: loop {
			let tile_min = (x, y);
			let tile_max = (
				(x + settings.tile_size.0).min(settings.camera_settings.backbuffer_width),
				(y + settings.tile_size.1).min(settings.camera_settings.backbuffer_height),
			);
			let width = tile_max.0 - tile_min.0;
			let height = tile_max.1 - tile_min.1;

			initial_tiles.push(Tile {
				sample_count: 0,
				left: tile_min.0,
				top: tile_min.1,
				width,
				height,
				data: vec![Vector3::new(0.0, 0.0, 0.0); width * height],
			});

			y += settings.tile_size.1;
			if y >= settings.camera_settings.backbuffer_height {
				y = 0;
				x += settings.tile_size.0;
			}
			if x >= settings.camera_settings.backbuffer_width {
				break 'gen_tiles;
			}
		}
	}

	let queue = Arc::new(ArrayQueue::new(initial_tiles.len()));
	for t in initial_tiles {
		queue.push(t).unwrap();
	}

	let thread_count = Arc::new(AtomicUsize::new(settings.worker_count));
	for _ in 0..settings.worker_count {
		let queue = queue.clone();
		let sender = sender.clone();

		// We clone both our scene and settings
		// This simplifies implementation
		let context = TraceContext {
			scene: scene.clone(),
			settings: settings.clone(),
		};

		let thread_count = thread_count.clone();
		thread::spawn(move || loop {
			let mut tile = match queue.pop() {
				Some(t) => t,
				None => {
					if 1 == thread_count.fetch_sub(1, Ordering::Relaxed) {
						sender.send(Message::Finished);
					}
					return;
				}
			};

			for y in tile.top..(tile.top + tile.height) {
				for x in tile.left..(tile.left + tile.width) {
					let primary = generate_primary_ray(x, y, &context.settings.camera_settings);
					let sample = trace(primary, &context, 1);

					// Map the global pixel indices to the local tile buffer and store the sample
					tile.data[(x - tile.left) + (y - tile.top) * tile.width] += sample;
				}
			}

			tile.sample_count += 1;
			info!(target: "TraceCore", "Tile[{:4}, {:4}] finished sample [{}]", tile.left, tile.top, tile.sample_count);

			// Check if we are done
			if tile.sample_count == context.settings.sample_count {
				sender.send(Message::TileFinished(tile.clone())).unwrap();
			} else {
				queue.push(tile.clone());

				// Check if we want to send our tile down the pipe
				if context.settings.samples_per_iteration != 0 && tile.sample_count % context.settings.samples_per_iteration == 0 {
					sender.send(Message::TileProgressed(tile.clone())).unwrap();
				}
			}
		});
	}

	return TaskHandle {
		receiver,
		settings,
		alive_thread_count: thread_count,
		callback: None,
	};
}

fn trace(ray: Ray, context: &TraceContext, depth: usize) -> Vector3 {
	let settings = &context.settings;

	if depth > settings.bounce_limit {
		return Vector3::new(0.0, 0.0, 0.0);
	}

	let intersect = context.scene.intersect(ray);
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

	let view_dir = (settings.camera_settings.transform.position - fragment_position).normalize();
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
			context,
			depth + 1,
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
			context,
			depth + 1,
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

fn generate_primary_ray(x: usize, y: usize, camera: &CameraSettings) -> Ray {
	let width = camera.backbuffer_width as f32;
	let height = camera.backbuffer_height as f32;
	let aspect = width / height;
	let x = x as f32 + (rand::random::<f32>() - 0.5);
	let y = y as f32 + (rand::random::<f32>() - 0.5);

	let px = (2.0 * ((x + 0.5) / width) - 1.0) * f32::tan(camera.fov_vert / 2.0 * PI / 180.0) * aspect;
	let py = (1.0 - 2.0 * ((y + 0.5) / height)) * f32::tan(camera.fov_vert / 2.0 * PI / 180.0);

	Ray::new(camera.transform.position, Vector3::new(px, py, 1.0).normalize())
}

fn generate_primary_ray_with_dof(x: usize, y: usize, camera: &CameraSettings) -> Ray {
	let primary = generate_primary_ray(x, y, camera);

	// chose random point on the aperture, through rejection sampling
	let start = loop {
		let r1 = rand::random::<f32>() * 2.0 - 1.0;
		let r2 = rand::random::<f32>() * 2.0 - 1.0;
		let ax = camera.transform.position.x + r1 * camera.aperture_radius;
		let ay = camera.transform.position.y + r2 * camera.aperture_radius;
		let _start = Vector3::new(ax, ay, camera.transform.position.z);
		if _start.distance(camera.transform.position) < camera.aperture_radius {
			break _start;
		}
	};

	let focal_plane = Plane {
		origin: camera.transform.position + Vector3::new(0.0, 0.0, 1.0) * camera.focal_length,
		normal: Vector3::new(0.0, 0.0, -1.0),
	};
	let end = camera.transform.position + focal_plane.intersects(primary).unwrap().distance * primary.direction;

	return Ray {
		origin: start,
		direction: (end - start).normalize(),
	};
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
