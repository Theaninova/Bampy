use bvh::bvh::BvhNode;

use super::{mesh::Mesh, slice_path::SlicePath, z_projection::ToolpathIntersects, FloatValue};

pub fn trace_surface(slice: &mut SlicePath, surface: &Mesh, a: FloatValue) {
    slice.points.retain_mut(|point| {
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
                    if child_l_aabb.toolpath_intersects(point, a) {
                        stack.push(child_l_index);
                    }
                    if child_r_aabb.toolpath_intersects(point, a) {
                        stack.push(child_r_index);
                    }
                }
                BvhNode::Leaf {
                    parent_index: _,
                    shape_index,
                } => {
                    let triangle = &surface.triangles[shape_index];
                    macro_rules! check {
                        ( $var:ident ) => {{
                            let x = point.x - triangle.$var.x;
                            let y = point.y - triangle.$var.y;
                            (point.z > triangle.aabb.min.z
                                && FloatValue::sqrt(x * x + y * y)
                                    < (triangle.$var.z - point.z).abs() * a)
                        }};
                    }
                    if check!(a) || check!(b) || check!(c) {
                        return false;
                    }
                }
            }
        }
        true
    });
}
