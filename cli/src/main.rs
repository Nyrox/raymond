use raytracer::{acc_grid, mesh::Mesh, scene::Light};

use core::{prelude::*, primitives::*, Scene, SceneObject, Settings};

use cgmath::Vector3;
use std::cell::RefCell;

use raytracer::trace;

use std::{
	path::PathBuf,
	sync::{
		atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT},
		Arc,
		RwLock,
	},
	time::{Duration, Instant},
};

use std::fs::File;

use std::io::prelude::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cli")]
enum Opt {
	#[structopt(name = "dump-default-config")]
	DumpConfig {
		#[structopt(short = "o", long, default_value = "default-config.ron")]
		out: PathBuf,
	},
	#[structopt(name = "render")]
	Render {
		#[structopt(default_value = "./", short = "s", long)]
		scene: PathBuf,
		#[structopt(short = "c", long)]
		config: Option<PathBuf>,
	},
}

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
	use simplelog::*;
	CombinedLogger::init(vec![WriteLogger::new(
		LevelFilter::Info,
		Config::default(),
		File::create("trace.log").unwrap(),
	)]);

	match Opt::from_args() {
		Opt::DumpConfig { out } => {
			let mut f = File::create(&out)?;
			let settings = core::settings::Settings::default();
			let settings = ron::ser::to_string_pretty(&settings, Default::default())?;

			f.write(settings.as_bytes())?;
		}
		Opt::Render { config, scene } => {
			// Load the tracer settings
			let settings = match config {
				Some(ref p) => Settings::load(p)?,
				None => Settings::default(),
			};

			// Try and load the scene file
			let scene = match &scene {
				_ if scene.is_file() => Scene::load(&scene)?,
				_ if scene.is_dir() => {
					unimplemented!("Loading scenes from a directory is not yet supported")
				}
				// There probably exists some realm where this is the case
				_ => panic!("Expected [context] to be a file or a directory"),
			};

			let now = Instant::now();

			let task_handle = trace::render_tiled(scene, settings.clone());
			let render = task_handle.r#await();

			let final_image = render;

			fn element_wise_map<Fun: Fn(f64) -> f64>(vec: &Vector3<f64>, f: Fun) -> Vector3<f64> {
				Vector3::new(f(vec.x), f(vec.y), f(vec.z))
			}

			let mut export = vec![
				Vector3::new(0, 0, 0);
				settings.camera.backbuffer_width
					* settings.camera.backbuffer_height
			];
			for (i, p) in final_image.iter().enumerate() {
				let exposure = 1.0;
				let gamma = 2.2;
				let tone_mapped = Vector3::new(1.0, 1.0, 1.0)
					- element_wise_map(&(p * -1.0 * exposure), |e| f64::exp(e));
				let tone_mapped = element_wise_map(&tone_mapped, |x| x.powf(1.0 / gamma));

				for i in 0..3 {
					let e = tone_mapped[i];
					if e > 1.0 || e < 0.0 {
						println!("Problem: {:?}", e);
					}
				}

				match (tone_mapped * 255.0).cast() {
					Some(v) => {
						export[i] = v;
					}
					None => println!("PROBLEM: {:?}", tone_mapped * 255.0),
				};
			}

			println!(
				"Finished render.\nTotal render time: {}s\n",
				(Instant::now() - now).as_millis() as f32 / 1000.0,
			);

			let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_fn(
				settings.camera.backbuffer_width as u32,
				settings.camera.backbuffer_height as u32,
				|x, y| {
					image::Rgb(
						export[(x + y * settings.camera.backbuffer_width as u32) as usize].into(),
					)
				},
			);
			image
				.save("output.png")
				.expect("Failed to save buffer to disk");
		}
	};

	return Ok(());
}
