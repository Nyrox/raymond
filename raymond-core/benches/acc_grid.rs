use criterion::{black_box, criterion_group, criterion_main, Criterion};
use raymond_core::{geometry::{AccGrid, Mesh, Ray}, Vector3};





fn criterion_benchmark(c: &mut Criterion) {
    let dragon_grid = AccGrid::build_from_mesh(Mesh::load_ply(include_str!("../../assets/meshes/dragon_vrip.ply")));

    c.bench_function("Random Rays at Stanford Dragon", |b| b.iter_batched(
        || {
            let ray = Ray::random_direction(Vector3::new(0.0, 0.0, 0.0));
            Ray { 
                direction: ray.direction,
                origin: ray.direction * -100.0, // move away from the center
            }
        },
        |ray| dragon_grid.intersects(ray),
        criterion::BatchSize::SmallInput,
    ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);