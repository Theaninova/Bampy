use std::fmt::Debug;

use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::BHShape,
    bvh::Bvh,
};
use nalgebra::{Point, Scalar, SimdPartialOrd};

pub mod base_slices;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Triangle<T: Scalar, const D: usize> {
    pub a: Point<T, D>,
    pub b: Point<T, D>,
    pub c: Point<T, D>,
    pub normal: Point<T, D>,
    pub node_index: usize,
}

impl<T: SimdPartialOrd + Scalar + Copy, const D: usize> Bounded<T, D> for Triangle<T, D> {
    fn aabb(&self) -> Aabb<T, D> {
        let mut aabb = self.a.aabb();
        aabb.grow_mut(&self.b);
        aabb.grow_mut(&self.c);
        aabb
    }
}

impl<T: SimdPartialOrd + Scalar + Copy, const D: usize> BHShape<T, D> for Triangle<T, D> {
    fn set_bh_node_index(&mut self, node_index: usize) {
        self.node_index = node_index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Line<T: Scalar, const D: usize> {
    pub start: Point<T, D>,
    pub end: Point<T, D>,
}

#[derive(Debug)]
pub struct SlicerOptions {
    pub aabb: bvh::aabb::Aabb<f32, 3>,
    pub bvh: Bvh<f32, 3>,
    pub triangles: Vec<Triangle<f32, 3>>,
    pub layer_height: f32,
}
