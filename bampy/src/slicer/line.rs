use nalgebra::{Scalar, Vector3};

/// Lines are assumed to be with respect to the XY plane.
/// The order of the points is therefore significant,
/// clockwise order with respect to the inside of the shape,
/// meaning the inside is on the right hand side of the line.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Line3<T: Scalar> {
    pub start: Vector3<T>,
    pub end: Vector3<T>,
}
