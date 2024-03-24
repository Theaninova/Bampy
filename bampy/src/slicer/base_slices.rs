use super::{
    line::Line3,
    mesh::Mesh,
    slice_rings::{find_slice_rings, SliceRing},
    FloatValue, SlicerOptions,
};
use bvh::bvh::BvhNode;

#[derive(Debug)]
pub struct BaseSlice {
    pub z: FloatValue,
    pub lines: Vec<Line3>,
}

/// Creates base slices from the geometry, excluding surfaces.
/// The slicse are not sorted or separated into rings.
pub fn create_slices(options: &SlicerOptions, slicable: &Mesh) -> Vec<SliceRing> {
    let layer_count = (slicable.aabb.max.z / options.layer_height).floor() as usize;
    let mut rings = vec![];
    let mut layer_index = 0;

    for i in 0..layer_count {
        let layer = i as FloatValue * options.layer_height;
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
                    assert!(child_l_aabb.min.z <= child_l_aabb.max.z);
                    assert!(child_r_aabb.min.z <= child_r_aabb.max.z);
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

        rings.append(&mut find_slice_rings(base_slice, &mut layer_index));
    }

    rings
}
