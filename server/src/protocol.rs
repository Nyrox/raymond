use serde::{Deserialize, Serialize};

use raytracer::Tile;

///
///
///

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
	TileProgressed(Tile),
	TileFinished(Tile),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Command {}
