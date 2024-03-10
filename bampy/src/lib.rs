use bvh::{aabb::Bounded, bvh::Bvh};
use nalgebra::Point;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::slicer::{base_slices, SlicerOptions, Triangle};

mod slicer;
mod util;

#[wasm_bindgen]
pub fn slice(positions: &[f32], face_normals: &[f32], layer_height: f32) {
    debug_assert_eq!(positions.len() % 9, 0);
    debug_assert_eq!(face_normals.len() % 3, 0);
    debug_assert_eq!(positions.len() / 9, face_normals.len() / 3);

    let mut triangles = Vec::with_capacity(positions.len() / 9);
    for i in (0..positions.len()).step_by(9) {
        let triangle = Triangle {
            a: Point::<f32, 3>::new(positions[i], positions[i + 1], positions[i + 2]),
            b: Point::<f32, 3>::new(positions[i + 3], positions[i + 4], positions[i + 5]),
            c: Point::<f32, 3>::new(positions[i + 6], positions[i + 7], positions[i + 8]),
            normal: Point::<f32, 3>::new(
                face_normals[i / 9],
                face_normals[i / 9 + 1],
                face_normals[i / 9 + 2],
            ),
            node_index: 0,
        };
        triangles.push(triangle);
    }

    let mut aabb = triangles[0].aabb();
    for triangle in &triangles {
        aabb.grow_mut(&triangle.a);
        aabb.grow_mut(&triangle.b);
        aabb.grow_mut(&triangle.c);
    }

    let slicer_options = SlicerOptions {
        aabb,
        bvh: Bvh::build(&mut triangles),
        triangles,
        layer_height,
    };
    let base_slices = base_slices::create_base_slices(&slicer_options, &vec![]);
}
