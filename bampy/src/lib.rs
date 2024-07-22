#![feature(extract_if)]
use std::collections::VecDeque;

use approx::relative_eq;
use nalgebra::{point, vector, Vector3};
use num::Float;
use result::{Slice, SliceOptions, SliceResult};
use slicer::{axis::Axis, slice_path::SlicePath, trace_surface::trace_surface};
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
        let mut normal = triangle.normal.clone();
        normal.z = normal.z.abs();
        let angle = normal.angle(&BED_NORMAL);
        if angle <= max_angle || relative_eq!(angle, max_angle) {
            surface_triangles.push(triangle);
        }
    }
    slicable_triangles.shrink_to_fit();
    surface_triangles.shrink_to_fit();

    console_log!("Creating Surfaces");
    let min_surface_area = std::f64::consts::PI * (nozzle_diameter / 2.0).powi(2);
    let mut surfaces = split_surface(surface_triangles)
        .into_iter()
        .filter(|mesh| {
            let mut surface_area = 0.0;
            for triangle in &mesh.triangles {
                surface_area += triangle.area();
                if surface_area >= min_surface_area {
                    return true;
                }
            }
            return false;
        })
        .map(|mesh| {
            let outline = mesh
                .outline_base_slice(Axis::Z)
                .find_paths()
                .into_iter()
                .filter(|path| path.closed)
                .collect::<Vec<_>>();
            let surface = mesh
                .slice_surface(Axis::X, nozzle_diameter)
                .filter(|path| {
                    let mut length = 0.0;
                    for pair in path.path.windows(2) {
                        length += (pair[0].coords - pair[1].coords).norm();
                        if length >= min_surface_path_length {
                            return true;
                        }
                    }
                    return false;
                })
                .collect::<Vec<_>>();
            (mesh, outline, surface)
        })
        .collect::<Vec<_>>();
    surfaces
        .sort_unstable_by(|(a, _, _), (b, _, _)| a.aabb.min.z.partial_cmp(&b.aabb.min.z).unwrap());

    console_log!("Creating Walls");
    let wallMesh = Mesh::from(slicable_triangles);
    let mut walls = wallMesh
        .slice_paths(Axis::Z, layer_height)
        .flat_map(|paths| paths.into_iter().filter(|path| path.closed))
        .collect::<VecDeque<_>>();
    let mut active_surfaces = Vec::new();
    let mut out = Vec::new();

    console_log!("Resolving dependencies");
    while let Some(mut wall) = walls.pop_front() {
        active_surfaces.extend(
            surfaces
                .extract_if(|surface| surface.0.aabb.min.z <= wall.aabb.max.z)
                .map(|surface| (surface, Vec::new())),
        );

        let deactivate =
            active_surfaces.extract_if(|element| element.0 .0.aabb.max.z < wall.aabb.min.z);
        for (surface, surface_walls) in deactivate {
            for ring in surface.1 {
                out.push(ring.points);
            }
            for path in surface.2 {
                out.push(path.path);
            }
            for wall in surface_walls {
                walls.push_front(wall);
            }
        }

        for surface in active_surfaces.iter_mut() {
            let held = wall
                .points
                .extract_if(|point| !trace_surface(&point, &surface.0 .0, max_angle))
                .collect::<Vec<_>>();
            if !held.is_empty() {
                surface.1.push(SlicePath {
                    points: held,
                    ..wall
                });
            }
        }

        if !wall.points.is_empty() {
            out.push(wall.points);
        }
    }

    console_log!("Done");
    SliceResult {
        slices: out
            .into_iter()
            .map(|slice| Slice::Ring {
                position: slice
                    .into_iter()
                    .flat_map(|point| [point.x as f32, point.y as f32, point.z as f32])
                    .collect(),
            })
            .collect(),
    }
    /*SliceResult {
        slices: surfaces
            .into_iter()
            .flat_map(|(_, outlines, slices)| {
                outlines
                    .into_iter()
                    .map(|slice| Slice::Ring {
                        position: slice
                            .points
                            .into_iter()
                            .flat_map(|point| [point.x as f32, point.y as f32, point.z as f32])
                            .collect(),
                    })
                    .chain(slices.into_iter().map(|slice| {
                        Slice::Ring {
                            position: slice
                                .path
                                .into_iter()
                                .flat_map(|point| [point.x as f32, point.y as f32, point.z as f32])
                                .collect(),
                        }
                    }))
            })
            .chain(walls.flatten().map(|slice| {
                Slice::Ring {
                    position: slice
                        .points
                        .into_iter()
                        .flat_map(|point| [point.x as f32, point.y as f32, point.z as f32])
                        .collect(),
                }
            }))
            .collect(),
    }*/
}
