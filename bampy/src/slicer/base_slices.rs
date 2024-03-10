use bvh::bvh::BvhNode;

use crate::console_log;

use super::{
    line::Line3,
    mesh::Mesh,
    slice_rings::{find_slice_rings, SliceRing},
    SlicerOptions,
};

#[derive(Debug)]
pub struct BaseSlice {
    pub z: f32,
    pub lines: Vec<Line3<f32>>,
}

/// Creates base slices from the geometry, excluding surfaces.
/// The slicse are not sorted or separated into rings.
pub fn create_slices(options: &SlicerOptions, slicable: &Mesh<f32>) -> Vec<SliceRing> {
    let layer_count = f32::floor(slicable.aabb.max.z / options.layer_height) as usize;
    let mut rings = vec![];

    for i in 0..layer_count {
        console_log!("Layer {}", i);
        let layer = i as f32 * options.layer_height;
        let mut base_slice = BaseSlice {
            z: layer,
            lines: vec![],
        };

        let mut stack = Vec::<usize>::with_capacity(slicable.bvh.nodes.len());
        stack.push(0);
        while let Some(i) = stack.pop() {
            match slicable.bvh.nodes[i] {
                BvhNode::Node {
                    parent_index: _,
                    child_l_index,
                    child_l_aabb,
                    child_r_index,
                    child_r_aabb,
                } => {
                    if layer >= child_l_aabb.min.z && layer <= child_l_aabb.max.z {
                        stack.push(child_l_index);
                    }
                    if layer >= child_r_aabb.min.z && layer <= child_r_aabb.max.z {
                        stack.push(child_r_index);
                    }
                }
                BvhNode::Leaf {
                    parent_index: _,
                    shape_index,
                } => {
                    slicable.triangles[shape_index]
                        .intersect_z(layer)
                        .map(|line| {
                            base_slice.lines.push(line);
                        });
                }
            }
        }

        rings.append(&mut find_slice_rings(base_slice));
    }

    rings
}
