use nalgebra::Vector3;

use super::FloatValue;

/// Lines are assumed to be with respect to the XY plane.
/// The order of the points is therefore significant,
/// clockwise order with respect to the inside of the shape,
/// meaning the inside is on the right hand side of the line.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Line3 {
    pub start: Vector3<FloatValue>,
    pub end: Vector3<FloatValue>,
}
