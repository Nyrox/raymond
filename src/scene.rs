use cgmath::*;
use cgmath::prelude::*;
use raytracer::{Ray};
use acc_grid;

type F = f64;
const F_MAX: F = ::std::f64::MAX;

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub distance: F,
    pub ray: Ray,
    pub subobject_index: usize,
}

impl Hit {
    pub fn new(ray: Ray, distance: F) -> Hit {
        Hit { ray, distance, subobject_index: 0 }
    }

    pub fn with_child(ray: Ray, distance: F, subobject_index: usize) -> Hit {
        Hit { ray, distance, subobject_index }
    }
}

#[derive(Debug, Clone)]
pub enum Material {
    Diffuse(Vector3<F>, F),
    Metal(Vector3<F>, F),
    Emission(Vector3<F>),
}

#[derive(Clone, Debug)]
pub struct SurfaceProperties {
    pub normal: Vector3<F>
}

#[derive(Clone)]
pub enum Object {
    Plane(Plane),
    Sphere(Sphere),
    Model(Model),
    Grid(Arc<acc_grid::AccGrid>, Material),
}

impl Object {
    pub fn get_material(&self) -> &Material {
        match self {
            &Object::Plane(ref p) => &p.material,
            &Object::Sphere(ref s) => &s.material,
            &Object::Model(ref m) => &m.material,
            &Object::Grid(ref g, ref m) => m,
        }
    }

    pub fn intersects(&self, ray: Ray) -> Option<Hit> {
         match self {
            &Object::Plane(ref p) => p.intersects(ray),
            &Object::Sphere(ref s) => s.intersects(ray),
            &Object::Model(ref m) => m.intersects(ray),
            &Object::Grid(ref g, _) => g.intersects(ray),
        }
    }

    pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
        match self {
            &Object::Plane(ref p) => p.get_surface_properties(hit),
            &Object::Sphere(ref s) => s.get_surface_properties(hit),
            &Object::Model(ref m) => m.get_surface_properties(hit),
            &Object::Grid(ref g, _) => g.get_surface_properties(hit),
        }
    }
}

use std::sync::Arc;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Model {
    pub mesh: Arc<Mesh>,
    pub material: Material,
}

impl Model {
    pub fn intersects(&self, ray: Ray) -> Option<Hit> {
        self.mesh.intersects(ray)
    }
    pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
        self.mesh.get_surface_properties(hit)
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: Vector3<F>,
    pub normal: Vector3<F>,
    pub uv: Vector2<F>,
    pub tangent: Vector3<F>,
}

impl Vertex {
	pub fn calculate_tangent(x: Vertex, y: Vertex, z: Vertex) -> Vector3<F> {
		let edge1 = y.position - x.position;
		let edge2 = z.position - x.position;

		let uv1 = y.uv - x.uv;
		let uv2 = z.uv - x.uv;

		let f = 1.0 / (uv1.x * uv2.y - uv2.x * uv1.y);
		let mut tangent = Vector3::new(0.0, 0.0, 0.0);
		tangent.x = f * (uv2.y * edge1.x - uv1.y * edge2.x);
		tangent.y = f * (uv2.y * edge1.y - uv1.y * edge2.y);
		tangent.z = f * (uv2.y * edge1.z - uv1.y * edge2.z);

		tangent.normalize()
	}
}

#[derive(Clone, Debug)]
pub struct AABB {
    pub min: Vector3<F>,
    pub max: Vector3<F>,
}

use std::mem;

impl AABB {
    pub fn intersects(&self, ray: Ray) -> Option<Hit> {
        let inverse_ray_dir = 1.0 / ray.direction;
        let mut t1 = (self.min[0] - ray.origin[0]) * inverse_ray_dir[0];
        let mut t2 = (self.max[0] - ray.origin[0]) * inverse_ray_dir[0];

        let mut tmin = t1.min(t2);
        let mut tmax = t1.max(t2);

        for i in 1..3 {
            t1 = (self.min[i] - ray.origin[i]) * inverse_ray_dir[i];
            t2 = (self.max[i] - ray.origin[i]) * inverse_ray_dir[i];

            tmin = tmin.max(t1.min(t2));
            tmax = tmax.min(t1.max(t2));
        }

        if !(tmax > tmin.max(0.0)) { return None; }
        
        return Some(Hit::new(ray, tmin));
    }
}

#[derive(Debug)]
pub struct Triangle(Vertex, Vertex, Vertex);

impl Triangle {
    pub fn intersects(&self, ray: Ray) -> Option<Hit> {
        const EPSILON: F = 0.00000001;

        let (vertex0, vertex1, vertex2) = 
            (self.0.position, self.1.position, self.2.position);

        let edge1 = vertex1 - vertex0;
        let edge2 = vertex2 - vertex0;

        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);
        if (a < EPSILON && a > -EPSILON) { return None; }


        let f = 1.0 / a;
        let s = ray.origin - vertex0;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 { return None; }

        let q = s.cross(edge1);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || u + v > 1.0 { return None; }

        let t = f * edge2.dot(q);
        if t > EPSILON {
            return Some(Hit::new(ray, t));
        }

        return None;
    }

    pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
        fn area(a: Vector3<F>, b: Vector3<F>, c: Vector3<F>) -> F {
            let ab = a.distance(b);
            let ac = a.distance(c);
            let bc = b.distance(c);
            let s = (ab + ac + bc) / 2.0;
            return (
                s * (s - ab) * (s - ac) * (s - bc)
            ).sqrt();
        }

        let position = hit.ray.origin + hit.ray.direction * hit.distance;
        let abc = area(self.0.position, self.1.position, self.2.position);
        let abp = area(self.0.position, self.1.position, position);
        let bcp = area(self.0.position, self.2.position, position);
        let ba = abp / abc;
        let bb = bcp / abc;
        let bc = 1.0 - (ba + bb);
        let normal = 
            (self.2.normal * ba) +
            (self.1.normal * bb) + 
            (self.0.normal * bc);

        return SurfaceProperties {
            normal
        };
    }

    pub fn find_bounds(&self) -> AABB {
        let mut min = Vector3::<F>::new(125125.0, 1251251.0, 12512512.0);
        let mut max = Vector3::<F>::new(-123125.0, -125123.0, -512123.0);

        for i in 0..3 {
            min[i] = min[i].min(self.0.position[i]);
            min[i] = min[i].min(self.1.position[i]);
            min[i] = min[i].min(self.2.position[i]);
            max[i] = max[i].max(self.0.position[i]);
            max[i] = max[i].max(self.1.position[i]);
            max[i] = max[i].max(self.2.position[i]);
        }
        
        return AABB { min, max };
    }
}


#[derive(Debug)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
    pub bounding_box: AABB,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Mesh { bounding_box: Self::find_mesh_bounds(&triangles), triangles}
    }

    pub fn intersects(&self, ray: Ray) -> Option<Hit> {
        self.bounding_box.intersects(ray)?;
        let mut closest = F_MAX;
        let mut closest_hit = None;

        for (i, tri) in self.triangles.iter().enumerate() {
            match tri.intersects(ray) {
                Some(h) => { 
                    let distance = h.distance;
                    if distance < closest {
                        closest = distance;
                        closest_hit = Some(Hit::with_child(ray, closest, i));
                    }
                }
                None => {}
            }
        }

        closest_hit
    }

    pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
        self.triangles[hit.subobject_index].get_surface_properties(hit)
    }

    pub fn bake_transform(&mut self, translate: Vector3<F>) {
        for mut triangle in self.triangles.iter_mut() {
            triangle.0.position += translate;
            triangle.1.position += translate;
            triangle.2.position += translate;
        }

        self.bounding_box = Self::find_mesh_bounds(&self.triangles);
    }

    pub fn load_ply(path: PathBuf) -> Mesh {
		let buffer = fs::read_to_string(path).unwrap();

		let mut vertices = Vec::<Vertex>::new();
		let mut faces = Vec::<Triangle>::new();
		let mut lines = buffer.lines();

		// Parse header
		'header: while let Some(line) = lines.next() {
			let mut tokens = line.split_whitespace();

			match tokens.next().unwrap() {
				"element" => {
					match tokens.next().unwrap() {
						"vertex" => vertices.reserve_exact(tokens.next().unwrap().parse::<usize>().unwrap()),
						_ => { }
					}
				}
				"end_header" => break 'header,
				_ => { }
			}
		};

		// Parse vertices
		for _ in 0..vertices.capacity() {
			let mut line = lines.next().unwrap();
			let mut tokens = line.split_whitespace();
			let values = tokens.map(|t| t.parse::<F>().unwrap()).collect::<Vec<F>>();
			vertices.push(Vertex {
				position: Vector3::new(values[0], values[1], values[2]),
				normal: Vector3::new(values[3], values[4], values[5]),
				uv: Vector2::new(*values.get(6).unwrap_or(&0.0), *values.get(7).unwrap_or(&0.0)),
				tangent: Vector3::new(0.0, 0.0, 0.0)
			});
		};

		// Parse faces
		'faces: while let Some(line) = lines.next() {
			let mut tokens = line.split_whitespace();
			let values = tokens.map(|t| t.parse::<u32>().unwrap()).collect::<Vec<u32>>();

			match values[0] {
				3 => {
					let mut face = [values[1], values[2], values[3]];

					let tangent = Vertex::calculate_tangent(vertices[face[0] as usize].clone(), vertices[face[1] as usize].clone(), vertices[face[2] as usize].clone());
					vertices[face[0] as usize].tangent = tangent;
					vertices[face[1] as usize].tangent = tangent;
					vertices[face[2] as usize].tangent = tangent;

                    faces.push(Triangle(
                        vertices[values[1] as usize].clone(),
                        vertices[values[2] as usize].clone(),
                        vertices[values[3] as usize].clone(),
                    ));
				}
				_ => { }
			}
		};

		Mesh::new(faces)
    }

    pub fn find_mesh_bounds(tris: &Vec<Triangle>) -> AABB {
        let mut min = Vector3::<F>::new(125125.0, 1251251.0, 12512512.0);
        let mut max = Vector3::<F>::new(-123125.0, -125123.0, -512123.0);

        for tri in tris {
            for i in 0..3 {
                min[i] = min[i].min(tri.0.position[i]);
                min[i] = min[i].min(tri.1.position[i]);
                min[i] = min[i].min(tri.2.position[i]);

                max[i] = max[i].max(tri.0.position[i]);
                max[i] = max[i].max(tri.1.position[i]);
                max[i] = max[i].max(tri.2.position[i]);
            }
        }
        
        return AABB { min, max };
    }

}


#[derive(Clone)]
pub struct Plane {
    pub origin: Vector3<F>,
    pub normal: Vector3<F>,
    pub material: Material,
}

impl Plane {
    pub fn get_material(&self) -> Material {
        self.material.clone()
    }

    pub fn intersects(&self, ray: Ray) -> Option<Hit> {
        let normal = self.normal;

        let denom = normal.dot(-ray.direction);
        if denom > 1e-6 {
            let p0l0 = self.origin - ray.origin;
            let t = p0l0.dot(-normal) / denom;
            if t >= 0.0 {
                return Some(Hit::new(ray, t));
            }
        }

        return None;
    }

    pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
        SurfaceProperties {
            normal: self.normal
        }
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub origin: Vector3<F>,
    pub radius: F,
    pub material: Material,
}

impl Sphere {
    fn get_material(&self) -> Material {
        self.material.clone()
    }
    
    fn intersects(&self, ray: Ray) -> Option<Hit> {
        let c = self.origin - ray.origin;
        let mut t = c.dot(ray.direction);
        let q = c - t * ray.direction;
        let p = q.dot(q);
        
        if p > self.radius * self.radius {
            return None;
        }

        t -= (self.radius * self.radius - p).sqrt();
        if t <= 0.0 { return None; }
        
        let hit_pos = ray.origin + ray.direction * t;
        return Some(Hit::new(ray, t));
    }

    pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
        SurfaceProperties {
            normal: ((hit.ray.origin + hit.ray.direction * hit.distance) - self.origin).normalize()
        }
    }
}

#[derive(Clone)]
pub struct Light {
    pub position: Vector3<F>,
    pub intensity: Vector3<F>,
}

#[derive(Clone)]
pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene { objects: Vec::new(), lights: Vec::new() }
    }
}