use bvh::bvh::BvhNode;
use nalgebra::Point;

use super::{Line, SlicerOptions};

#[derive(Debug)]
pub struct BaseSlice {
    z: f32,
    lines: Vec<Line<f32, 3>>,
}

/**
 * Creates base slices from the geometry, excluding surfaces.
 *
 * The slicse are not sorted or separated into rings.
 */
pub fn create_base_slices(options: &SlicerOptions, surface_triangles: &[bool]) -> Vec<BaseSlice> {
    let layer_count = f32::floor(options.aabb.max.z / options.layer_height) as usize;
    let mut base_slices = Vec::<BaseSlice>::with_capacity(layer_count);

    for i in 0..layer_count {
        let layer = i as f32 * options.layer_height;
        let base_slice = BaseSlice {
            z: layer,
            lines: vec![],
        };

        let mut stack = Vec::<usize>::with_capacity(options.bvh.nodes.len());
        stack.push(0);
        while let Some(i) = stack.pop() {
            match options.bvh.nodes[i] {
                BvhNode::Node {
                    parent_index: _,
                    child_l_index,
                    child_l_aabb,
                    child_r_index,
                    child_r_aabb,
                } => {
                    if layer >= child_l_aabb.min.z && layer <= child_l_aabb.max.z {
                        stack.push(child_r_index);
                    }
                    if layer >= child_r_aabb.min.z && layer <= child_r_aabb.max.z {
                        stack.push(child_l_index);
                    }
                }
                BvhNode::Leaf {
                    parent_index: _,
                    shape_index,
                } => {
                    let triangle = options.triangles[shape_index];
                    let a = Point::<f32, 3>::new(triangle.a.x, triangle.a.y, layer);
                }
            }
        }

        base_slices.push(base_slice);
    }

    base_slices
}
