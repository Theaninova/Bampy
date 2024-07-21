use approx::relative_eq;
use nalgebra::{point, vector, Vector3};
use result::{Slice, SliceOptions, SliceResult};
use slicer::axis::Axis;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::slicer::{mesh::Mesh, split_surface::split_surface, triangle::Triangle, FloatValue};

mod result;
mod slicer;
mod util;

const BED_NORMAL: Vector3<f64> = vector![0f64, 0f64, 1f64];

#[wasm_bindgen]
pub fn slice(
    SliceOptions {
        positions,
        layer_height,
        max_angle,
        min_surface_path_length,
        nozzle_diameter,
    }: SliceOptions,
) -> SliceResult {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    assert_eq!(positions.len() % 9, 0);

    let mut surface_triangles = Vec::<Triangle>::with_capacity(positions.len() / 9);
    let mut slicable_triangles = Vec::<Triangle>::with_capacity(positions.len() / 9);
    for i in (0..positions.len()).step_by(9) {
        let triangle = Triangle::new(
            point![
                positions[i] as FloatValue,
                positions[i + 1] as FloatValue,
                positions[i + 2] as FloatValue
            ],
            point![
                positions[i + 3] as FloatValue,
                positions[i + 4] as FloatValue,
                positions[i + 5] as FloatValue
            ],
            point![
                positions[i + 6] as FloatValue,
                positions[i + 7] as FloatValue,
                positions[i + 8] as FloatValue
            ],
        );

        slicable_triangles.push(triangle);
        let angle = triangle.normal.angle(&BED_NORMAL);
        let opposite_angle = std::f64::consts::PI - angle;
        if angle <= max_angle
            || relative_eq!(angle, max_angle)
            || opposite_angle <= max_angle
            || relative_eq!(opposite_angle, max_angle)
        {
            surface_triangles.push(triangle);
        }
    }
    slicable_triangles.shrink_to_fit();
    surface_triangles.shrink_to_fit();

    console_log!("Creating Surfaces");
    let surfaces = split_surface(surface_triangles).into_iter().map(|mesh| {
        mesh.slice_surface(Axis::X, nozzle_diameter).filter(|path| {
            let mut length = 0.0;
            for pair in path.path.windows(2) {
                length += (pair[0].coords - pair[1].coords).norm();
                if length >= min_surface_path_length {
                    return true;
                }
            }
            return false;
        })
    });

    console_log!("Creating Walls");
    let wallMesh = Mesh::from(slicable_triangles);
    let walls = wallMesh.slice_paths(Axis::Z, layer_height);

    /*console_log!("Tracing Surfaces");
    let a = max_angle.tan();
    for slice in &mut slices {
        for surface in &surfaces {
            if surface.aabb.min.z <= slice.z && surface.aabb.max.z > slice.z {
                trace_surface(slice, surface, a);
            }
        }
    }*/

    console_log!("Done");
    SliceResult {
        slices: surfaces
            .flatten()
            .map(|slice| Slice::Ring {
                position: slice
                    .path
                    .into_iter()
                    .flat_map(|point| [point.x as f32, point.y as f32, point.z as f32])
                    .collect(),
            })
            /*.chain(walls.flatten().map(|slice| {
                Slice::Ring {
                    position: slice
                        .points
                        .into_iter()
                        .flat_map(|point| [point.x as f32, point.y as f32, point.z as f32])
                        .collect(),
                }
            }))*/
            /*.chain(surfaces.into_iter().map(|surface| {
                Slice::Surface {
                    position: surface
                        .triangles
                        .into_iter()
                        .flat_map(|triangle| {
                            [
                                triangle.a.x as f32,
                                triangle.a.y as f32,
                                triangle.a.z as f32,
                                triangle.b.x as f32,
                                triangle.b.y as f32,
                                triangle.b.z as f32,
                                triangle.c.x as f32,
                                triangle.c.y as f32,
                                triangle.c.z as f32,
                            ]
                        })
                        .collect(),
                }
            }))*/
            .collect(),
    }
}
