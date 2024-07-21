use super::{mesh::Mesh, triangle::Triangle};
use bvh::bvh::{Bvh, BvhNode};

/// Splits a surface into connected surfaces.
/// TODO: self intersections
pub fn split_surface(mut triangles: Vec<Triangle>) -> Vec<Mesh> {
    let mut surfaces = vec![];
    while let Some(triangle) = triangles.pop() {
        let mut surface = vec![triangle];
        let mut bvh = Bvh::build(&mut surface);
        let mut aabb = surface[0].aabb.clone();

        loop {
            let mut mutated = false;
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

        if surface.len() > 1 {
            surfaces.push(Mesh {
                triangles: surface,
                aabb,
                bvh,
            })
        }
    }
    surfaces
}
