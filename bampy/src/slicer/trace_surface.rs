use bvh::bvh::BvhNode;
use nalgebra::Point3;

use crate::slicer::sdf::Sdf3dModifiers;

use super::{
    mesh::Mesh,
    sdf::{Sdf, SdfInfiniteCone},
    z_projection::ToolpathIntersects,
    FloatValue,
};

pub fn trace_surface(point: &Point3<FloatValue>, surface: &Mesh, a: FloatValue) -> bool {
    let sdf = SdfInfiniteCone::new(a);
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
                if sdf.translate(triangle.a).sdf(point) < 0.0
                    || sdf.translate(triangle.b).sdf(point) < 0.0
                    || sdf.translate(triangle.c).sdf(point) < 0.0
                {
                    return false;
                }
            }
        }
    }
    return true;
}
