use scene::Scene;
use num_traits;
use cgmath::prelude::*;
use cgmath::*;
use std::cell::RefCell;
use rand;

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

pub fn element_wise_map<Fun: Fn(F) -> F>(vec: &Vector3<F>, f: Fun) -> Vector3<F> {
    Vector3::new(f(vec.x), f(vec.y), f(vec.z))
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

                self.image[x + y * self.width] = self.trace(&ray, 2);
            }
        }
    }

    fn trace(&mut self, ray: &Ray, bounces: usize) -> Vector3<F> {
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
        
        let scene = self.scene.borrow();
        let object = &scene.objects[closest.1];
        let hit = closest.2;

        let mut total = Vector3::new(0.0, 0.0, 0.0);
        
        // Direct lighting
        'light_iter: for light in scene.lights.iter() {
            // Shadow
            for (index, shadow_object) in scene.objects.iter().enumerate() {
                if closest.1 == index { continue; }
                let shadow_hit = shadow_object.check_ray(&Ray { origin: light.position, direction: (hit.position - light.position).normalize() });
                if let Some(shadow_hit) = shadow_hit {
                    if shadow_hit.position.distance(light.position) < light.position.distance(hit.position) {
                        continue 'light_iter;
                    }
                }
            }

            let distance = light.position.distance(hit.position);
            let attenuation = 1.0 / (distance * distance);
            let cos_theta = hit.normal.dot((light.position - hit.position).normalize()).max(0.0);
            
            total += light.intensity * cos_theta * attenuation;
        }

        if bounces == 0 { return total.mul_element_wise(object.get_material().color) / PI; }

        // Indirect lighting
        const N: usize = 4;

        let mut total_indirect = Vector3::new(0.0, 0.0, 0.0);
        let local_cartesian = self.create_coordinate_system_of_n(hit.normal);
        let local_cartesian_transform = Matrix3::from_cols(local_cartesian.0, hit.normal, local_cartesian.1);
        for i in 0..N {
            let sample = self.uniform_sample_hemisphere();
            let sample_world = local_cartesian_transform * sample;

            total_indirect += sample_world.dot(hit.normal).max(0.0) * self.trace(&Ray { origin: hit.position, direction: sample_world }, bounces - 1);
        }
        total_indirect /= N as F * (1.0 / (2.0 * PI));

        return (total + total_indirect).mul_element_wise(object.get_material().color) / PI;
    }

    fn uniform_sample_hemisphere(&self) -> Vector3<F> {
        let r1 = rand::random::<F>();
        let r2 = rand::random::<F>();

        let sin_theta = (1.0 - r1 * r1).sqrt();
        let phi = 2.0 * PI * r2;
        let x = sin_theta * phi.cos();
        let z = sin_theta * phi.sin();
        return Vector3::new(x, r1, z);
    }
    
    fn create_coordinate_system_of_n(&self, n: Vector3<F>) -> (Vector3<F>, Vector3<F>) {
        let nt = Vector3::new(n.z, 0.0, -n.x).normalize();
        let nb = n.cross(nt);

        return (nt, nb);
    }
    

    // Exports the internal HDR format to RGB for saving or display
    pub fn export_image(&self) -> Vec<Vector3<u8>> {
        let mut export: Vec<Vector3<u8>> = vec!(Vector3::new(0, 0, 0); (self.width * self.height) as usize);

        for (i, p) in self.image.iter().enumerate() {
            // let tone_mapped = element_wise_division(p , &(p + Vector3::new(1.0, 1.0, 1.0)));
            let exposure = 1.0;
            let gamma = 2.2;
            let tone_mapped = Vector3::new(1.0, 1.0, 1.0) - element_wise_map(&(p * -1.0 * exposure), |e| F::exp(e));
            let tone_mapped = element_wise_map(&tone_mapped, |x| x.powf(1.0 / gamma));
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

        Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(px, py, 1.0).normalize())
    }
}