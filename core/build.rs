use genfut::{genfut, Opt};

fn main() {
    genfut(Opt {
        name: "bbox_intersect".to_string(),
        file: std::path::PathBuf::from("gpu/bbox_intersect.fut"),
        author: "Nyrox <root@nyrox.dev>".to_string(),
        version: "0.1.0".to_string(),
        license: "WTF".to_string(),
        description: "".to_string(),
    })
}