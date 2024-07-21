use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(from_wasm_abi)]
pub struct SliceOptions {
    #[tsify(type = "Float32Array")]
    pub positions: Vec<f32>,
    pub layer_height: f64,
    pub nozzle_diameter: f64,
    pub max_angle: f64,
    pub min_surface_path_length: f64,
}

#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
#[tsify(into_wasm_abi)]
pub enum Slice {
    Surface {
        #[tsify(type = "Float32Array")]
        position: Vec<f32>,
    },
    Ring {
        #[tsify(type = "Float32Array")]
        position: Vec<f32>,
    },
    Path {
        #[tsify(type = "Float32Array")]
        position: Vec<f32>,
    },
}

#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub struct SliceResult {
    pub slices: Vec<Slice>,
}
