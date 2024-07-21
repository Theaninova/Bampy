use nalgebra::{Point3, Vector3};

use super::FloatValue;

/// Lines are assumed to be with respect to the XY plane.
/// The order of the points is therefore significant,
/// clockwise order with respect to the inside of the shape,
/// meaning the inside is on the right hand side of the line.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Line3 {
    pub start: Point3<FloatValue>,
    pub end: Point3<FloatValue>,
}

impl Line3 {
    pub fn norm(&self) -> FloatValue {
        (self.end - self.start).norm()
    }

    pub fn normal(&self) -> Vector3<FloatValue> {
        (self.end - self.start).normalize()
    }
}
