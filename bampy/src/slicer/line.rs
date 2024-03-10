use nalgebra::{Scalar, Vector3};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Line3<T: Scalar> {
    pub start: Vector3<T>,
    pub end: Vector3<T>,
}
