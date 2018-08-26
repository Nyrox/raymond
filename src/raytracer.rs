use scene::Scene;
use num_traits;
use cgmath::prelude::*;
use cgmath::{self, Vector3};
use std::cell::RefCell;

type F = f32;
const PI: F = 3.14;

pub struct Ray {
    pub origin: Vector3<F>,
    pub direction: Vector3<F>,
}

#[derive(Debug)]
pub struct Hit {
    pub position: Vector3<F>,
    pub normal: Vector3<F>,
}

impl Default for Hit {
    fn default() -> Hit {
        Hit { position: Vector3::new(0.0, 0.0, 0.0), normal: Vector3::new(0.0, 0.0, 0.0) }
    }
}

impl Ray {
    pub fn new (origin: Vector3<F>, direction: Vector3<F>) -> Ray {
        Ray { origin, direction }
    }
}


pub fn element_wise_division(left: &Vector3<F>, right: &Vector3<F>) -> Vector3<F> {
    Vector3::new(left.x / right.x, left.y / right.y, left.z / right.z)
}

pub struct Raytracer<'a> {
    pub image: Vec<Vector3<F>>,
    pub width: usize,
    pub height: usize,
    pub fov: F,
    pub scene: &'a RefCell<Scene>,
}


impl<'a> Raytracer<'a> {
    pub fn new(width: usize, height: usize, fov: F, scene: &'a RefCell<Scene>) -> Raytracer<'a> {
        Raytracer {
            image: vec!(Vector3::new(0.0, 0.0, 0.0); (width * height) as usize),
            width, height,
            fov,
            scene
        }
    }

    pub fn render(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let ray = self.generate_primary_ray(x, y);

                self.image[x + y * self.width] = self.trace(&ray);
            }
        }
    }

    fn trace(&mut self, ray: &Ray) -> Vector3<F> {
        let mut closest: (F, usize, Hit) = (123125.0, 12958125, Hit::default());

        for (i, object) in self.scene.borrow().objects.iter().enumerate() {
            match object.check_ray(ray) {
                Some(hit) => { 
                    let distance = ray.origin.distance(hit.position);
                    if distance < closest.0 {
                        closest = (distance, i, hit);
                    }
                }
                None => continue,
            }
        }

        if closest.1 == 12958125 { return Vector3::new(0.0, 0.0, 0.0) };
        
        println!("HIT!");

        let scene = self.scene.borrow();
        let object = &scene.objects[closest.1];
        let hit = closest.2;

        let mut total = Vector3::new(0.0, 0.0, 0.0);
        
        for light in scene.lights.iter() {
            let cos_theta = hit.normal.dot((light.position - hit.position).normalize()).max(0.0);
            
            total += light.intensity.mul_element_wise(object.get_material().color) * cos_theta;
        }

        return total;
    }

    // Exports the internal HDR format to RGB for saving or display
    pub fn export_image(&self) -> Vec<Vector3<u8>> {
        let mut export: Vec<Vector3<u8>> = vec!(Vector3::new(0, 0, 0); (self.width * self.height) as usize);

        for (i, p) in self.image.iter().enumerate() {
            let tone_mapped = element_wise_division(p , &(p + Vector3::new(1.0, 1.0, 1.0)));
            export[i] = (tone_mapped * 255.0).cast().unwrap();
        }

        return export;
    }

    fn generate_primary_ray(&self, x: usize, y: usize) -> Ray {
        let width = self.width as f32; let height = self.height as f32;
        let x = x as f32; let y = y as f32;
        let aspect = width / height;

        let px = (2.0 * ((x + 0.5) / width) - 1.0) * F::tan(self.fov / 2.0 * PI / 180.0) * aspect;
        let py = (1.0 - 2.0 * ((y + 0.5) / height)) * F::tan(self.fov / 2.0 * PI / 180.0);

        Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(px, py, -1.0))
    }
}