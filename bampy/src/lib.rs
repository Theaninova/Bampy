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

const BED_NORMAL: Vector3<f32> = vector![0f32, 0f32, 1f32];

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

    let mut surface_triangles = Vec::<Triangle<f32>>::with_capacity(positions.len() / 9);
    let mut slicable_triangles = Vec::with_capacity(positions.len() / 9);
    for i in (0..positions.len()).step_by(9) {
        let triangle = Triangle::new(
            vector![positions[i], positions[i + 1], positions[i + 2]],
            vector![positions[i + 3], positions[i + 4], positions[i + 5]],
            vector![positions[i + 6], positions[i + 7], positions[i + 8]],
        );

        if triangle.normal.angle(&BED_NORMAL) > max_angle {
            slicable_triangles.push(triangle);
        } else {
            slicable_triangles.push(triangle);
            //surface_triangles.push(triangle);
        }
    }
    slicable_triangles.shrink_to_fit();
    surface_triangles.shrink_to_fit();

    let slicer_options = SlicerOptions { layer_height };

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
                    .flat_map(|point| [point.x, point.y, point.z])
                    .collect()
            })
            .collect(),
    }
}
