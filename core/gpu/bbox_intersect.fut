
type Vec3 = (f32, f32, f32)
type Ray = { origin: Vec3, direction: Vec3 }
type BBox = {}

entry bbox_intersect (x: []i32) (y: []i32): i32 =
  reduce (+) 0 (map2 (*) x y)

