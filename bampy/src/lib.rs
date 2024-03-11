use nalgebra::{vector, Vector3};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::slicer::{
    base_slices::create_slices, mesh::Mesh, split_surface::split_surface, triangle::Triangle,
    SlicerOptions,
};

mod slicer;
mod util;

const BED_NORMAL: Vector3<f64> = vector![0f64, 0f64, 1f64];

#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(from_wasm_abi)]
pub struct SliceOptions {
    #[tsify(type = "Float32Array")]
    positions: Vec<f32>,
    layer_height: f32,
    max_angle: f32,
}

#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub struct SliceResult {
    #[tsify(type = "Array<Float32Array>")]
    rings: Vec<Vec<f32>>,
}

#[wasm_bindgen]
pub fn slice(
    SliceOptions {
        positions,
        layer_height,
        max_angle,
    }: SliceOptions,
) -> SliceResult {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    assert_eq!(positions.len() % 9, 0);

    let mut surface_triangles = Vec::<Triangle<f64>>::with_capacity(positions.len() / 9);
    let mut slicable_triangles = Vec::<Triangle<f64>>::with_capacity(positions.len() / 9);
    for i in (0..positions.len()).step_by(9) {
        let triangle = Triangle::new(
            vector![
                positions[i] as f64,
                positions[i + 1] as f64,
                positions[i + 2] as f64
            ],
            vector![
                positions[i + 3] as f64,
                positions[i + 4] as f64,
                positions[i + 5] as f64
            ],
            vector![
                positions[i + 6] as f64,
                positions[i + 7] as f64,
                positions[i + 8] as f64
            ],
        );

        if triangle.normal.angle(&BED_NORMAL) > max_angle as f64 {
            slicable_triangles.push(triangle);
        } else {
            slicable_triangles.push(triangle);
            //surface_triangles.push(triangle);
        }
    }
    slicable_triangles.shrink_to_fit();
    surface_triangles.shrink_to_fit();

    let slicer_options = SlicerOptions {
        layer_height: layer_height as f64,
    };

    console_log!("Creating Surfaces");
    // let surfaces = split_surface(surface_triangles);

    console_log!("Computing BVH");
    let slicable = Mesh::from(slicable_triangles);
    console_log!("Creating Slices");
    let slices = create_slices(&slicer_options, &slicable);
    console_log!("Done");

    SliceResult {
        rings: slices
            .into_iter()
            .map(|slice| {
                slice
                    .points
                    .into_iter()
                    .flat_map(|point| [point.x as f32, point.y as f32, point.z as f32])
                    .collect()
            })
            .collect(),
    }
}
