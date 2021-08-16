

entry bbox_intersect (x: []i32) (y: []i32): i32 =
  reduce (+) 0 (map2 (*) x y)
