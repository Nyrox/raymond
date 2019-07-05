use std::{
	fs::{self, File},
	io::{Read, Write},
	path::Path,
};

use core::Scene;

#[test]
fn scene_load_and_store() {
	let input_path = format!(
		"{}/tests/resources/basic_scene/scene.ron",
		env!("CARGO_MANIFEST_DIR")
	);
	let output_path = format!(
		"{}/tests/resources/basic_scene/scene_parsed.ron",
		env!("CARGO_MANIFEST_DIR")
	);

	let scene = Scene::load(&input_path).expect("failed to load scene");
	scene.store(&output_path).expect("failed to save scene");

	let new_scene = Scene::load(&output_path).expect("failed to load scene");

	assert_eq!(scene, new_scene);

	fs::remove_file(&output_path).expect("failed to remove file");
}
