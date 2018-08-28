#![feature(duration_as_u128)]
#![feature(nll)]

extern crate num_traits;
extern crate cgmath;
extern crate image;
extern crate rand;
extern crate crossbeam_utils;

pub mod raytracer;
pub mod scene;
use scene::*;
use raytracer::*;
use cgmath::Vector3;
use std::cell::RefCell;

use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::time::{Duration, Instant};
use std::sync::RwLock;
use std::sync::Arc;

static TRACE_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
static SHADOW_RAY_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
static SHADOW_TOTAL_TIME: AtomicUsize = ATOMIC_USIZE_INIT;

fn main() {
    let WIDTH = 600;
    let HEIGHT = 400;

    let now = Instant::now();

    let mut scene = Scene::new();
    scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(1.0, -0.5, 2.5), radius: 0.5, material: Material::Diffuse(
        Vector3::new(1.0, 0.00, 0.00), 0.6
    )}));
    scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(-1.25, -0.25, 3.5), radius: 0.75, material: Material::Metal(
        Vector3::new(0.0, 0.25, 1.00), 0.15
    )}));
    scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(-0.1, -0.65, 2.2), radius: 0.35, material: Material::Metal(
        Vector3::new(1.0, 1.0, 0.0), 0.3,
    )}));

    // // Floor
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(0.0, -1.0, 0.0), normal: Vector3::new(0.0, 1.0, 0.0), material: Material::Diffuse(
        Vector3::new(0.75, 0.75, 0.75), 0.5
    )}));
    // Ceiling
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(0.0, 2.0, 0.0), normal: Vector3::new(0.0, -1.0, 0.0), material: Material::Emission(
        Vector3::new(0.9, 0.9, 0.9),
    )}));
    // Backwall
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(0.0, 0.0, 5.0), normal: Vector3::new(0.0, 0.0, -1.0), material: Material::Diffuse(
        Vector3::new(1.0, 1.0, 1.0), 0.5
    )}));
    // left wall
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(-2.0, 0.0, 0.0), normal: Vector3::new(1.0, 0.0, 0.0), material: Material::Diffuse(
        Vector3::new(0.0, 0.0, 1.0), 0.5
    )}));
    // right wall
    scene.objects.push(Object::Plane(Plane { origin: Vector3::new(2.0, 0.0, 0.0), normal: Vector3::new(-1.0, 0.0, 0.0), material: Material::Diffuse(
        Vector3::new(0.0, 1.0, 0.0), 0.5
    )}));
    
    // scene.lights.push(Light { position: Vector3::new(0.0, 1.95, 2.5), intensity: Vector3::new(0.8, 0.8, 1.0) });
    // scene.lights.push(Light { position: Vector3::new(1.75, -0.75, 1.0), intensity: Vector3::new(0.8, 1.0, 0.7) });

    let mut raytracer = Raytracer::new(WIDTH, HEIGHT, 75.0, scene.clone());
    raytracer.render();

    println!("Finished render.\nTotal render time: {}s\nTotal amount of trace calls: {}\nTotal amount of shadow rays cast: {}\n", 
        (Instant::now() - now).as_millis() as f32 / 1000.0,
        TRACE_COUNT.load(Ordering::Relaxed),
        SHADOW_RAY_COUNT.load(Ordering::Relaxed),
    );
    println!("Total time spent on shadow rays: {}s", SHADOW_TOTAL_TIME.load(Ordering::Relaxed) as f32 / 1000.0 / 1000.0 / 1000.0);

    let rgb: Vec<[u8; 3]> = raytracer.export_image().into_iter().map(|p| p.into()).collect();
    let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x, y| {
        image::Rgb(rgb[(x + y * WIDTH as u32) as usize])
    });
    image.save("output.png").expect("Failed to save buffer to disk");
}