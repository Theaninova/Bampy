use std::collections::LinkedList;

use nalgebra::{vector, Vector3};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::slicer::{
    base_slices::create_base_slices, split_surface::split_surface, Mesh, SlicerOptions, Triangle,
};

mod slicer;
mod util;

const BED_NORMAL: Vector3<f32> = vector![0f32, 0f32, 1f32];

#[wasm_bindgen]
pub fn slice(positions: &[f32], layer_height: f32, max_angle: f32) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    assert_eq!(positions.len() % 9, 0);

    let mut surface_triangles = LinkedList::<Triangle<f32>>::new();
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
            surface_triangles.push_back(triangle);
        }
    }
    slicable_triangles.shrink_to_fit();

    console_log!("Computing BVH");

    let slicer_options = SlicerOptions { layer_height };

    console_log!("Creating Surfaces");
    let surfaces = split_surface(surface_triangles);

    console_log!("Creating Slices");
    let slicable = Mesh::from(slicable_triangles);
    let base_slices = create_base_slices(&slicer_options, &slicable);
    console_log!("Done");
}
