use super::{line::Line3, mesh::Mesh, FloatValue, SlicerOptions};
use crate::console_log;
use bvh::bvh::BvhNode;

#[derive(Debug)]
pub struct BaseSlice {
    pub d: FloatValue,
    pub lines: Vec<Line3>,
}

/// Creates base slices from the geometry
pub fn create_base_slices<'a>(
    axis: usize,
    options: &'a SlicerOptions,
    slicable: &'a Mesh,
) -> impl Iterator<Item = BaseSlice> + 'a {
    let layer_count = ((slicable.aabb.max[axis] - slicable.aabb.min[axis]) / options.layer_height)
        .floor() as usize;

    (0..layer_count).map(move |i| {
        let layer = i as FloatValue * options.layer_height + slicable.aabb.min[axis];
        let mut base_slice = BaseSlice {
            d: layer,
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
                    assert!(child_l_aabb.min[axis] <= child_l_aabb.max[axis]);
                    assert!(child_r_aabb.min[axis] <= child_r_aabb.max[axis]);
                    if layer >= child_l_aabb.min[axis] && layer <= child_l_aabb.max[axis] {
                        stack.push(child_l_index);
                    }
                    if layer >= child_r_aabb.min[axis] && layer <= child_r_aabb.max[axis] {
                        stack.push(child_r_index);
                    }
                }
                BvhNode::Leaf {
                    parent_index: _,
                    shape_index,
                } => {
                    slicable.triangles[shape_index]
                        .intersect(layer, axis)
                        .map(|line| {
                            base_slice.lines.push(line);
                        });
                }
            }
        }

        base_slice
    })
}
