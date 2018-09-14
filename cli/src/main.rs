#![feature(duration_as_u128)]
#![feature(nll)]

extern crate raytracer;
extern crate cgmath;
extern crate image;

use raytracer::primitives::{Plane};
use raytracer::mesh::Mesh;
use raytracer::scene::{Scene, Object};
use raytracer::material::Material;
use raytracer::trace::*;
use raytracer::acc_grid;

use cgmath::Vector3;
use std::cell::RefCell;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::time::{Duration, Instant};
use std::sync::RwLock;
use std::sync::Arc;

static TRACE_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
static SHADOW_RAY_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
static SHADOW_TOTAL_TIME: AtomicUsize = ATOMIC_USIZE_INIT;

fn main() {
    let now = Instant::now();

    let mut scene = Scene::new();
    let mut sphere_mesh = Mesh::load_ply(PathBuf::from("assets/meshes/ico_sphere.ply"));


    // scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(-1.5, -0.5, 3.5), radius: 0.5, material: Material::Diffuse(
    //     Vector3::new(1.0, 0.00, 0.00), 0.04
    // )}));
    // scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(1.25, -0.25, 3.5), radius: 0.75, material: Material::Metal(
    //     Vector3::new(0.05, 0.25, 1.00), 0.02
    // )}));

    let mut cube_mesh = Mesh::load_ply(PathBuf::from("assets/meshes/dragon_vrip.ply"));
    cube_mesh.bake_transform(Vector3::new(0.0, -0.3, 2.9));
    // let mut cube_mesh = Arc::new(cube_mesh);
    let mut cube_grid = acc_grid::AccGrid::build_from_mesh(cube_mesh);
    // let hit = cube_grid.intersects(&Ray { origin: Vector3::new(0.0, 0.0, 0.0), direction: Vector3::new(0.0, 0.0, 1.0) });
    // println!("{:?}", hit);
    // panic!();
    // cube_grid.intersects(&Ray { origin: Vector3::new(0.0, 0.0, 0.0), direction: Vector3::new(0.0, 0.0, 1.0) });

    let cube_grid = Arc::new(cube_grid);
    let cube_model = Object::Grid(cube_grid, Material::Metal(
        Vector3::new(1.0, 1.0, 0.1), 0.15
    ));

    scene.objects.push(cube_model);


    // // Floor
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(0.0, -1.0, 0.0), normal: Vector3::new(0.0, 1.0, 0.0), material: Material::Diffuse(
        Vector3::new(0.75, 0.75, 0.75), 0.5
    )}));
    // Ceiling
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(0.0, 2.0, 0.0), normal: Vector3::new(0.0, -1.0, 0.0), material: Material::Emission(
        Vector3::new(1.5, 1.5, 1.5), Vector3::new(1.0, 1.0, 1.0), 0.27, 0.0
    )}));
    // Frontwall
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(0.0, 0.0, -2.0), normal: Vector3::new(0.0, 0.0, 1.0), material: Material::Diffuse(
        Vector3::new(1.0, 1.0, 1.0), 0.4
    )}));
    // Backwall
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(0.0, 0.0, 5.0), normal: Vector3::new(0.0, 0.0, -1.0), material: Material::Diffuse(
        Vector3::new(0.0, 0.0, 0.0), 0.9
    )}));
    // left wall
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(-2.0, 0.0, 0.0), normal: Vector3::new(1.0, 0.0, 0.0), material: Material::Diffuse(
        Vector3::new(0.0, 0.0, 0.0), 0.9
    )}));
    // right wall
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(2.0, 0.0, 0.0), normal: Vector3::new(-1.0, 0.0, 0.0), material: Material::Diffuse(
        Vector3::new(0.0, 0.0, 0.0), 0.9
    )}));


    // scene.lights.push(Light { position: Vector3::new(0.0, 1.95, 2.5), intensity: Vector3::new(0.8, 0.8, 1.0) });
    // scene.lights.push(Light { position: Vector3::new(1.75, -0.75, 1.0), intensity: Vector3::new(0.8, 1.0, 0.7) });
    let HEIGHT = 340;
    let WIDTH = HEIGHT / 9 * 16;

    let config = RaytracerConfig {
        width: WIDTH,
        height: HEIGHT,
        fov: 50.0,
        num_samples: 10,
        max_bounces: 5,

        ..RaytracerConfig::default()
    };

    let final_image: Vec<[u8; 3]> = config.launch(scene.clone()).into_iter().map(|p| p.into()).collect();


    println!("Finished render.\nTotal render time: {}s\nTotal amount of trace calls: {}\nTotal amount of shadow rays cast: {}\n", 
        (Instant::now() - now).as_millis() as f32 / 1000.0,
        TRACE_COUNT.load(Ordering::Relaxed),
        SHADOW_RAY_COUNT.load(Ordering::Relaxed),
    );
    println!("Total time spent on shadow rays: {}s", SHADOW_TOTAL_TIME.load(Ordering::Relaxed) as f32 / 1000.0 / 1000.0 / 1000.0);
    
    let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x, y| {
        image::Rgb(final_image[(x + y * WIDTH as u32) as usize])
    });
    image.save("output.png").expect("Failed to save buffer to disk");
}