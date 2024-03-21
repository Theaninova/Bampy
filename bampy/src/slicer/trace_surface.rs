use bvh::bvh::BvhNode;

use super::{mesh::Mesh, slice_rings::SliceRing};

pub fn trace_surface(slice: &mut SliceRing, surface: &Mesh<f64>) {
    loop {
        let mut mutated = false;
        slice.points.retain_mut(|triangle| {
            let mut stack = Vec::<usize>::new();
            stack.push(0);
            while let Some(i) = stack.pop() {
                match surface.bvh.nodes[i] {
                    BvhNode::Node {
                        parent_index: _,
                        child_l_index,
                        child_l_aabb,
                        child_r_index,
                        child_r_aabb,
                    } => {
                        if triangle.has_point_in_aabb(&child_l_aabb) {
                            stack.push(child_l_index);
                        }
                        if triangle.has_point_in_aabb(&child_r_aabb) {
                            stack.push(child_r_index);
                        }
                    }
                    BvhNode::Leaf {
                        parent_index: _,
                        shape_index,
                    } => {
                        if triangle.shares_point_with_triangle(surface[shape_index]) {
                            mutated = true;
                            surface.push(*triangle);
                            let index = surface.len() - 1;
                            bvh.add_shape(&mut surface, index);
                            aabb.join_mut(&triangle.aabb);
                            return false;
                        }
                    }
                }
            }
            true
        });
        if !mutated {
            break;
        }
    }
}
