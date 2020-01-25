pub mod watch;

use std::path::PathBuf;
use structopt::StructOpt;

use core::project;
use core::geometry;
use core::math::prelude::*;
use core::Material;

use raytracer::{transform::Transform, trace::{CameraSettingsBuilder, SettingsBuilder}};

#[derive(StructOpt, Debug)]
#[structopt(name = "raymond-cli")]
enum RaymondCli {
	Render {
		#[structopt(long)]
		watch: bool,
		#[structopt(name = "SCENE")]
		scene: PathBuf,
	},
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = RaymondCli::from_args();
	println!("{:?}", cli);




	match cli {
		RaymondCli::Render { watch, scene } => {
			println!("{:?}, {:?}", watch, scene);

	let project = project::Project::load(scene)?;
			let scene = project.build_scene();

				let HEIGHT = 720;
	let WIDTH = 1280;

	let camera = CameraSettingsBuilder::default()
		.backbuffer_width(WIDTH)
		.backbuffer_height(HEIGHT)
		.fov_vert(55.0)
		.transform(Transform::identity())
		.focal_length(2.5)
		.aperture_radius(0.0)
		.build()
		.unwrap();

	let settings = SettingsBuilder::default()
		.camera_settings(camera)
		.sample_count(25)
		.tile_size((128, 128))
		.bounce_limit(5)
		.samples_per_iteration(3)
		.build()
		.unwrap();

			let handle = raytracer::trace::render_tiled(scene, settings);


			if watch {
				let mut watcher = watch::start_watcher(1280, 720)?;

				'wait: loop {
					watcher.update();

					if watcher.wants_to_close() {
						break 'wait;
					}

					while let Some(tile) = handle.poll() {
						match tile {
							raytracer::trace::Message::TileProgressed(tile) => {
								watcher.send_tile(tile);
							},
							_ => ()
						}
					}
				}
			} else {
			}
		}
		_ => unimplemented!(),
	}

	Ok(())
}
