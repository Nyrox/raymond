use std::path::Path;

use plots::rays;

pub mod plots;

fn main() {
	std::fs::create_dir_all(Path::new("visuals/rays")).unwrap();

	rays::random_directions().unwrap();
}
