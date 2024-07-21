use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHValue;
use nalgebra::Point;

pub mod axis;
pub mod base_slices;
pub mod line;
pub mod mesh;
pub mod sdf;
pub mod slice_path;
pub mod split_surface;
pub mod trace_surface;
pub mod triangle;
pub mod z_projection;

pub type FloatValue = f64;

pub fn aabb_from_points<'a, I, T: BHValue, const D: usize>(mut points: I) -> Aabb<T, D>
where
    I: Iterator<Item = &'a Point<T, D>>,
{
    if let Some(point) = points.next() {
        let mut aabb = point.aabb();
        for point in points {
            aabb.grow_mut(point);
        }
        aabb
    } else {
        Aabb::empty()
    }
}
