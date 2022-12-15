use plotters::{prelude::*, style::SizeDesc};
use raymond_core::{
	prelude::{InnerSpace, Ray},
	Vector3, brdf::cook_torrance::{DefaultCookTorrance, CookTorrance},
};

const SAMPLE_SIZE: usize = 10000;

pub fn random_directions() -> Result<(), Box<dyn std::error::Error>> {
	let data = (0..SAMPLE_SIZE)
		.map(|_| Ray::random_direction(Vector3 { x: 0.0, y: 0.0, z: 0.0 }))
		.map(|ray| (ray.direction.x, ray.direction.y, ray.direction.z))
		.collect::<Vec<_>>();

	let root = BitMapBackend::gif("visuals/rays/random_directions.gif", (640, 480), 100)?.into_drawing_area();
	let mut chart = ChartBuilder::on(&root)
		.caption("Random directions around origin", ("sans-serif", 20))
		.build_cartesian_3d(-1.5..1.5f32, -1.5..1.5f32, -1.5..1.5f32)?;

	for yaw in 0..32 {
		root.fill(&WHITE)?;
		chart.with_projection(|mut p| {
			p.pitch = 1.57 - (1.57 - 20.0 as f64 / 50.0).abs();
			p.scale = 0.7;
			p.yaw = yaw as f64 / 32.0;
			p.into_matrix() // build the projection matrix
		});

		chart.configure_axes().light_grid_style(BLACK.mix(0.15)).max_light_lines(3).draw()?;

		for point in data.iter() {
			chart.draw_series(LineSeries::new([*point, (point.0 / 0.9, point.1 / 0.9, point.2 / 0.9)], &BLACK));
		}

		root.present()?;
	}

	root.present()?;
	Ok(())
}

pub fn random_directions_over_hemisphere() -> Result<(), Box<dyn std::error::Error>> {
	let data = (0..SAMPLE_SIZE)
		.map(|_| Ray::random_direction_over_hemisphere())
		.map(|ray| (ray.0.x, ray.0.y, ray.0.z))
		.collect::<Vec<_>>();

	let root = BitMapBackend::gif("visuals/rays/random_directions_over_hemisphere.gif", (640, 480), 100)?.into_drawing_area();
	let mut chart = ChartBuilder::on(&root)
		.caption("Random directions around origin", ("sans-serif", 20))
		.build_cartesian_3d(-1.5..1.5f32, -1.5..1.5f32, -1.5..1.5f32)?;

	for t in 0..32 {
		root.fill(&WHITE)?;
		chart.with_projection(|mut p| {
			p.pitch = 0.5 + (t as f64 / 80.0);
			p.scale = 0.7;
			p.yaw = (t + 5) as f64 / 32.0;
			p.into_matrix() // build the projection matrix
		});

		chart.configure_axes().light_grid_style(BLACK.mix(0.15)).max_light_lines(3).draw()?;

		for point in data.iter() {
			chart.draw_series(LineSeries::new([*point, *point], &BLACK))?;
		}

		root.present()?;
	}

	root.present()?;
	Ok(())
}

pub fn importance_sample_ggx_towards_camera() -> Result<(), Box<dyn std::error::Error>> {
	let root = BitMapBackend::gif("visuals/rays/importance_sample_ggx_towards_camera.gif", (640, 480), 100)?.into_drawing_area();
	let mut chart = ChartBuilder::on(&root)
		.caption("Random directions around origin", ("sans-serif", 20))
		.build_cartesian_3d(-1.5..1.5f32, -1.5..1.5f32, -1.5..1.5f32)?;

	let data = (0..SAMPLE_SIZE)
		.map(|_| DefaultCookTorrance::importance_sample(Vector3::new(-1.0, 1.0, -1.0).normalize(), 0.5))
		.map(|(ray, _)| (ray.x, ray.y, ray.z))
		.collect::<Vec<_>>();

	for t in 0..64 {
		root.fill(&WHITE)?;
		chart.with_projection(|mut p| {
			p.pitch = 0.5 + (-t as f64 / 80.0);
			p.scale = 0.7;
			p.yaw = (t + 5) as f64 / 32.0;
			p.into_matrix() // build the projection matrix
		});

		chart.configure_axes().light_grid_style(BLACK.mix(0.15)).max_light_lines(3).draw()?;

		for point in data.iter() {
			chart.draw_series(LineSeries::new([*point, *point], &BLACK))?;
		}

		root.present()?;
	}

	root.present()?;
	Ok(())
}
