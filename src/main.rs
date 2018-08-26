extern crate num_traits;
extern crate cgmath;
extern crate image;

pub mod raytracer;
pub mod scene;
use scene::*;
use raytracer::*;
use cgmath::Vector3;
use std::cell::RefCell;

fn main() {
    let WIDTH = 400;
    let HEIGHT = 300;

    let mut scene = Scene::new();
    scene.objects.push(Box::new(Sphere { origin: Vector3::new(1.0, 0.0, -2.0), radius: 0.5, material: Material {
        color: Vector3::new(0.6, 0.2, 0.1)
    }}));
    scene.lights.push(Light { position: Vector3::new(-0.5, 0.5, -0.5), intensity: Vector3::new(0.8, 0.8, 1.0) });
    scene.lights.push(Light { position: Vector3::new(1.75, -0.75, -1.0), intensity: Vector3::new(0.8, 1.0, 0.7) });

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