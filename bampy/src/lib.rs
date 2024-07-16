use std::iter::empty;

use approx::relative_eq;
use nalgebra::{vector, SimdBool, Vector3};
use serde::{Deserialize, Serialize};
use slicer::{line::Line3, slice_rings::slice_rings};
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::slicer::{
    base_slices::create_base_slices, mesh::Mesh, split_surface::split_surface,
    trace_surface::trace_surface, triangle::Triangle, FloatValue, SlicerOptions,
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
    layer_height: f64,
    max_angle: f64,
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
    slices: Vec<Slice>,
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

    let mut surface_triangles = Vec::<Triangle>::with_capacity(positions.len() / 9);
    let mut slicable_triangles = Vec::<Triangle>::with_capacity(positions.len() / 9);
    for i in (0..positions.len()).step_by(9) {
        let triangle = Triangle::new(
            vector![
                positions[i] as FloatValue,
                positions[i + 1] as FloatValue,
                positions[i + 2] as FloatValue
            ],
            vector![
                positions[i + 3] as FloatValue,
                positions[i + 4] as FloatValue,
                positions[i + 5] as FloatValue
            ],
            vector![
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

    let slicer_options = SlicerOptions { layer_height };

    console_log!("Creating Surfaces");
    let surfaces = split_surface(surface_triangles).into_iter().map(|mesh| {
        slice_rings(1, &slicer_options, &mesh)
            .flat_map(|mut rings| {
                /*let mut rings = rings
                .into_iter()
                .map(|mut ring| {
                    ring.points.sort_unstable_by(|a, b| {
                        a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    ring
                })
                .collect::<Vec<_>>();*/
                rings.sort_unstable_by(|a, b| {
                    a.points
                        .first()
                        .unwrap()
                        .partial_cmp(b.points.first().unwrap())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                rings
            })
            .fold(Vec::<Vector3<FloatValue>>::new(), |mut acc, mut curr| {
                if acc
                    .last()
                    .zip(curr.points.first())
                    .zip(curr.points.last())
                    .map_or(false, |((last, start), end)| {
                        start.metric_distance(last) > end.metric_distance(last)
                    })
                {
                    curr.points.reverse();
                }
                acc.extend(curr.points);
                acc
            })
    });

    console_log!("Computing BVH");
    let slicable = Mesh::from(slicable_triangles);
    console_log!("Creating Slices");
    let slices = slice_rings(2, &slicer_options, &slicable);

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
            .map(|slice| Slice::Ring {
                position: slice
                    .into_iter()
                    .flat_map(|point| [point.x as f32, point.y as f32, point.z as f32])
                    .collect(),
            })
            /*.chain(slices.flatten().map(|slice| {
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
