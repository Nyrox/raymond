use cgmath::*;
use cgmath::prelude::*;

use super::{F};

use super::primitives::{Ray, SurfaceProperties, Hit, AABB, Triangle};
use super::mesh::Mesh;


const GRID_DENSITY_BIAS: F = 3.0;

fn estimate_grid_resolution(bounds: &AABB, triangle_count: usize) -> Vector3<usize> {
    let size = bounds.max - bounds.min;
    let volume = (size.x * size.y * size.z).abs();

    let triangle_density = ((GRID_DENSITY_BIAS * triangle_count as F) / volume).powf(1.0 / 3.0);

    return Vector3::new(
        (size.x.abs() * triangle_density) as usize,
        (size.y.abs() * triangle_density) as usize,
        (size.z.abs() * triangle_density) as usize
    );
}

#[derive(Clone, Copy, Debug)]
pub struct Cell (usize);
// Naive repr. of Cell storing triangle indices directly
// Used to store an IR, for creating the effizient repr. later
#[derive(Clone, Debug)]
struct NaiveCell(Vec<usize>);

#[derive(Debug)]
pub struct AccGrid {
    pub cells: Vec<Cell>,
    pub mesh: Mesh,
    pub mapping_table: Vec<usize>,
    pub resolution: Vector3<usize>,
    pub cell_size: Vector3<F>,
}

impl AccGrid {
    pub fn build_from_mesh(mesh: Mesh) -> AccGrid {
        let grid_res = estimate_grid_resolution(&mesh.bounding_box, mesh.triangles.len());
        let cell_size = (mesh.bounding_box.max - mesh.bounding_box.min).div_element_wise(grid_res.cast::<F>().unwrap());
        let mut naive_cells = vec![NaiveCell(Vec::new()); grid_res.x * grid_res.y * grid_res.z];
        let mut mapping_table: Vec<usize> = Vec::new();

        for (index, tri) in mesh.triangles.iter().enumerate() {
            let bounds = tri.find_bounds();
            let mut cell_min = (bounds.min - mesh.bounding_box.min).div_element_wise(cell_size).cast::<usize>().expect("Failed to cast cell bounds to usize");
            let mut cell_max = (bounds.max - mesh.bounding_box.min).div_element_wise(cell_size).cast::<usize>().expect("Failed to cast cell bounds to usize");

            for i in 0..3 {
                cell_min[i] = cell_min[i].max(0).min(grid_res[i] - 1);
                cell_max[i] = cell_max[i].max(0).min(grid_res[i] - 1);
            }

            for z in cell_min.z ..= cell_max.z {
                for y in cell_min.y ..= cell_max.y {
                    for x in cell_min.x ..= cell_max.x {
                        naive_cells[x + grid_res.x * (y + z * grid_res.z)].0.push(index);
                    }
                }
            }
        }

        let mut cells = Vec::new();
        for c in naive_cells.iter() {
            cells.push(Cell(mapping_table.len()));
            mapping_table.push(c.0.len());
            for i in c.0.iter() {
                mapping_table.push(*i);
            }
        }

        AccGrid {
            cells,
            mesh,
            mapping_table,
            resolution: grid_res,
            cell_size
        }
    }

    pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
        self.mesh.triangles[hit.subobject_index].get_surface_properties(hit)
    }

    pub fn intersects(&self, ray: Ray) -> Option<Hit> {
        let outer_hit = self.mesh.bounding_box.intersects(ray)?;
        let outer_hit_position = ray.origin + ray.direction * outer_hit.distance;

        let mut start = ray.origin - self.mesh.bounding_box.min;
        let mut current_cell = start.div_element_wise(self.cell_size).cast::<i32>().unwrap();

        if current_cell.x < 0 || current_cell.y < 0 || current_cell.z < 0 { 
            start = outer_hit_position - self.mesh.bounding_box.min;
            current_cell = start.div_element_wise(self.cell_size).cast::<i32>().unwrap();
        }
        let step = Vector3::new(ray.direction.x.signum(), ray.direction.y.signum(), ray.direction.z.signum()).cast::<i32>().unwrap();

        let t_delta_x = if ray.direction.x < 0.0 { -self.cell_size.x } else { self.cell_size.x } / ray.direction.x;
        let t_delta_y = if ray.direction.y < 0.0 { -self.cell_size.y } else { self.cell_size.y } / ray.direction.y;
        let t_delta_z = if ray.direction.z < 0.0 { -self.cell_size.z } else { self.cell_size.z } / ray.direction.z;

        let mut t_max_x = (((current_cell.x + if ray.direction.x < 0.0 { 0 } else { 1 }) as f64 * self.cell_size.x) - start.x) / ray.direction.x;
        let mut t_max_y = (((current_cell.y + if ray.direction.y < 0.0 { 0 } else { 1 }) as f64 * self.cell_size.y) - start.y) / ray.direction.y;
        let mut t_max_z = (((current_cell.z + if ray.direction.z < 0.0 { 0 } else { 1 }) as f64 * self.cell_size.z) - start.z) / ray.direction.z;

        loop {
            let (x, y, z) = (current_cell.x as usize, current_cell.y as usize, current_cell.z as usize);
            if x + self.resolution.x * (y + z * self.resolution.z) >= self.cells.len() {
                return None;
            }

            let cell = self.cells[x + self.resolution.x * (y + z * self.resolution.z)];
            let count = self.mapping_table[cell.0];
            let mut closest = 5712515.0;
            let mut closest_hit = None;
            for i in 1..=count {
                let tri = &self.mesh.triangles[self.mapping_table[cell.0 + i]];
                match tri.intersects(ray) {
                    Some(h) => {
                        let distance = h.distance;
                        if distance < closest {
                            closest = distance;
                            closest_hit = Some(Hit::with_child(ray, distance, self.mapping_table[cell.0 + i]));
                        }
                    }
                    None => ()
                }
            }

            if closest_hit.is_some() { return closest_hit; }

            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    current_cell.x += step.x;
                    if current_cell.x >= self.resolution.x as i32 || current_cell.x < 0 {
                        return None;
                    }
                    t_max_x += t_delta_x;
                }
                else {
                    current_cell.z += step.z;
                    if current_cell.z >= self.resolution.z as i32 || current_cell.z < 0 {
                        return None;
                    }
                    t_max_z += t_delta_z;
                }
            }
            else {
                if t_max_y < t_max_z {
                    current_cell.y += step.y;
                    if current_cell.y >= self.resolution.y as i32 || current_cell.y < 0 {
                        return None;
                    }
                    t_max_y += t_delta_y;
                }
                else {
                    current_cell.z += step.z;
                    if current_cell.z >= self.resolution.z as i32 || current_cell.z < 0 {
                        return None;
                    }
                    t_max_z += t_delta_z;
                }
            }
        }
    }
}