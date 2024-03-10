use super::triangle::Triangle;
use bvh::bvh::{Bvh, BvhNode};

/// Splits a surface into connected surfaces.
pub fn split_surface(mut triangles: Vec<Triangle<f32>>) -> Vec<Vec<Triangle<f32>>> {
    let mut surfaces = vec![];
    while let Some(triangle) = triangles.pop() {
        let mut surface = vec![triangle];
        let mut bvh = Bvh::build(&mut surface);
        triangles.retain_mut(|triangle| {
            let mut stack = Vec::<usize>::new();
            stack.push(0);
            while let Some(i) = stack.pop() {
                match bvh.nodes[i] {
                    BvhNode::Node {
                        parent_index: _,
                        child_l_index,
                        child_l_aabb,
                        child_r_index,
                        child_r_aabb,
                    } => {
                        if triangle.intersects_aabb(&child_l_aabb) {
                            stack.push(child_l_index);
                        }
                        if triangle.intersects_aabb(&child_r_aabb) {
                            stack.push(child_r_index);
                        }
                    }
                    BvhNode::Leaf {
                        parent_index: _,
                        shape_index,
                    } => {
                        if triangle.connected_with_triangle(surface[shape_index]) {
                            surface.push(*triangle);
                            bvh.add_shape(&mut surface, surface.len() - 1);
                            return false;
                        }
                    }
                }
            }
            true
        })
    }
    surfaces
}
