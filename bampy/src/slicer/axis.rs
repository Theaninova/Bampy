use nalgebra::Vector3;

use super::FloatValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub fn other(&self) -> (Self, Self) {
        match self {
            Axis::X => (Axis::Y, Axis::Z),
            Axis::Y => (Axis::X, Axis::Z),
            Axis::Z => (Axis::X, Axis::Y),
        }
    }

    pub fn normal(&self) -> Vector3<FloatValue> {
        match self {
            Axis::X => Vector3::x(),
            Axis::Y => Vector3::y(),
            Axis::Z => Vector3::z(),
        }
    }
}
