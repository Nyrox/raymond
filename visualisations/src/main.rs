use std::path::Path;

use plots::rays;

pub mod plots;

fn main() {
	std::fs::create_dir_all(Path::new("visuals/rays")).unwrap();

	// rays::random_directions().unwrap();
	// rays::random_directions_over_hemisphere().unwrap();
	rays::importance_sample_ggx_towards_camera().unwrap();
	rays::visualise_pdf_over_hemisphere().unwrap();
}
