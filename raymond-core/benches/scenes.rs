use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use raymond_core::{
	geometry::{AccGrid, Mesh, Ray},
	project::Project,
	Vector3,
};

fn criterion_benchmark(c: &mut Criterion) {
	std::env::set_current_dir(Path::new(".."));

	let project = Project::parse(include_str!("../../assets/scenes/dragon_room/scene.json")).unwrap();
	let scene = project.build_scene();

    c.bench_function("Random Rays into Dragon Room Scene", |b| b.iter_batched(
        || Ray::random_direction(Vector3::new(0.0, 0.0, 0.0)),
        |ray| raymond_core::trace::trace(ray, &scene, 0, 5),
        criterion::BatchSize::SmallInput,
    ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
