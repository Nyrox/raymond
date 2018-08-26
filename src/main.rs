extern crate num_traits;
extern crate cgmath;
extern crate image;
extern crate rand;

pub mod raytracer;
pub mod scene;
use scene::*;
use raytracer::*;
use cgmath::Vector3;
use std::cell::RefCell;

fn main() {
    let WIDTH = 500;
    let HEIGHT = 330;



    let mut scene = Scene::new();
    scene.objects.push(Box::new(Sphere { origin: Vector3::new(1.0, -0.5, 2.5), radius: 0.5, material: Material {
        color: Vector3::new(1.0, 0.00, 0.00)
    }}));
    scene.objects.push(Box::new(Sphere { origin: Vector3::new(-1.0, -0.25, 3.5), radius: 0.75, material: Material {
        color: Vector3::new(0.0, 0.25, 1.00)
    }}));

    // Floor
    scene.objects.push(Box::new(Plane { origin: Vector3::new(0.0, -1.0, 0.0), normal: Vector3::new(0.0, 1.0, 0.0), material: Material {
        color: Vector3::new(1.0, 1.0, 1.0)
    }}));
    // Ceiling
    scene.objects.push(Box::new(Plane { origin: Vector3::new(0.0, 2.0, 0.0), normal: Vector3::new(0.0, -1.0, 0.0), material: Material {
        color: Vector3::new(1.0, 1.0, 1.0)
    }}));
    // Backwall
    scene.objects.push(Box::new(Plane { origin: Vector3::new(0.0, 0.0, 5.0), normal: Vector3::new(0.0, 0.0, -1.0), material: Material {
        color: Vector3::new(1.0, 1.0, 1.0)
    }}));
    // left wall
    scene.objects.push(Box::new(Plane { origin: Vector3::new(-2.0, 0.0, 0.0), normal: Vector3::new(1.0, 0.0, 0.0), material: Material {
        color: Vector3::new(0.0, 0.0, 1.0)
    }}));
    // right wall
    scene.objects.push(Box::new(Plane { origin: Vector3::new(2.0, 0.0, 0.0), normal: Vector3::new(-1.0, 0.0, 0.0), material: Material {
        color: Vector3::new(0.0, 1.0, 0.0)
    }}));
    
    scene.lights.push(Light { position: Vector3::new(0.0, 1.95, 2.5), intensity: Vector3::new(0.8, 0.8, 1.0) });
    // scene.lights.push(Light { position: Vector3::new(1.75, -0.75, 1.0), intensity: Vector3::new(0.8, 1.0, 0.7) });

    let mut scene = RefCell::new(scene);
    let mut raytracer = Raytracer::new(WIDTH, HEIGHT, 75.0, &scene);
    println!("{}", raytracer.image.len());
    raytracer.render();

    let rgb: Vec<[u8; 3]> = raytracer.export_image().into_iter().map(|p| p.into()).collect();
    let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x, y| {
        image::Rgb(rgb[(x + y * WIDTH as u32) as usize])
    });
    image.save("output.png").expect("Failed to save buffer to disk");
}