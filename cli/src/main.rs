pub mod watch;

use std::{path::PathBuf, time::Instant};
use structopt::StructOpt;

use raymond_core::{project, Vector3};

use raymond::{
	trace::{CameraSettingsBuilder, SettingsBuilder},
	transform::Transform,
};

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

			let project = project::Project::load_from_file(scene)?;
			let scene = project.build_scene();

			let HEIGHT = 720;
			let WIDTH = 1280;

			let camera = CameraSettingsBuilder::default()
				.backbuffer_width(WIDTH)
				.backbuffer_height(HEIGHT)
				.fov_vert(55.0)
				.transform(Transform { position: Vector3::new(0.0, 0.5, -1.0) })
				.focal_length(2.5)
				.aperture_radius(0.0)
				.build()
				.unwrap();

			let settings = SettingsBuilder::default()
				.camera_settings(camera)
				.sample_count(250)
				.tile_size((512, 512))
				.bounce_limit(5)
				.samples_per_iteration(5)
				.build()
				.unwrap();

			let start_time = Instant::now();
			let handle = raymond::trace::render_tiled(scene, settings);

			if watch {
				let mut watcher = watch::start(1280, 720)?;

				while watcher.is_open() {
					watcher.update();

					while let Some(tile) = handle.poll() {
						match tile {
							raymond::trace::Message::TileProgressed(tile) => {
								watcher.progress_tile(tile);
							}
							raymond::trace::Message::Finished => {
								println!("Render finished in: {} seconds", start_time.elapsed().as_secs_f32())
							}
							_ => (),
						}
					}
				}
				return Ok(());
			} else {
			}
		}
		_ => unimplemented!(),
	}

	Ok(())
}
