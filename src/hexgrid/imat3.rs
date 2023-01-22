use bevy::math::IVec3;
use std::ops::Mul;

/// Basic 3x3 matrix based on glam's Mat3 for f32 but using IVec3 to instead operate using integers.
#[derive(Clone, Copy, Debug)]
pub struct IMat3 {
    pub x_axis: IVec3,
    pub y_axis: IVec3,
    pub z_axis: IVec3,
}

impl IMat3 {
    pub const fn from_cols(x_axis: IVec3, y_axis: IVec3, z_axis: IVec3) -> Self {
        Self {
            x_axis,
            y_axis,
            z_axis,
        }
    }

    /// Transforms a 3D vector.
    pub fn mul_ivec3(&self, rhs: IVec3) -> IVec3 {
        self.x_axis.mul(rhs.x) + self.y_axis.mul(rhs.y) + self.z_axis.mul(rhs.z)
    }

    pub fn mul_imat3(&self, rhs: &Self) -> Self {
        Self::from_cols(
            self.mul(rhs.x_axis),
            self.mul(rhs.y_axis),
            self.mul(rhs.z_axis),
        )
    }
}

impl Mul<IVec3> for IMat3 {
    type Output = IVec3;
    fn mul(self, rhs: IVec3) -> Self::Output {
        self.mul_ivec3(rhs)
    }
}

impl Mul<IMat3> for IMat3 {
    type Output = IMat3;
    fn mul(self, rhs: IMat3) -> Self::Output {
        self.mul_imat3(&rhs)
    }
}
