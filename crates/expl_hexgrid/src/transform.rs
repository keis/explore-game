use super::{imat3::IMat3, HexCoord};
use glam::IVec3;

#[derive(Copy, Clone, Debug)]
pub enum Transform {
    Identity,
    RotateClockwise60,
    RotateClockwise120,
    RotateClockwise180,
    RotateClockwise240,
    RotateClockwise300,
    ReflectQ,
    ReflectR,
    ReflectS,
}

impl Transform {
    pub const fn apply(self, coord: HexCoord) -> HexCoord {
        match self {
            Transform::Identity => coord,
            Transform::RotateClockwise60 => HexCoord::new(-coord.r, -coord.s()),
            Transform::RotateClockwise120 => HexCoord::new(coord.s(), coord.q),
            Transform::RotateClockwise180 => HexCoord::new(-coord.q, -coord.r),
            Transform::RotateClockwise240 => HexCoord::new(coord.r, coord.s()),
            Transform::RotateClockwise300 => HexCoord::new(-coord.s(), -coord.q),
            Transform::ReflectQ => HexCoord::new(coord.q, coord.s()),
            Transform::ReflectR => HexCoord::new(coord.s(), coord.r),
            Transform::ReflectS => HexCoord::new(coord.r, coord.q),
        }
    }

    pub const fn into_imat3(self) -> IMat3 {
        match self {
            Transform::Identity => IMat3::from_cols(IVec3::X, IVec3::Y, IVec3::Z),
            Transform::RotateClockwise60 => IMat3::from_cols(
                IVec3::new(0, 0, -1),
                IVec3::new(-1, 0, 0),
                IVec3::new(0, -1, 0),
            ),
            Transform::RotateClockwise120 => IMat3::from_cols(
                IVec3::new(0, 1, 0),
                IVec3::new(0, 0, 1),
                IVec3::new(1, 0, 0),
            ),
            Transform::RotateClockwise180 => IMat3::from_cols(
                IVec3::new(-1, 0, 0),
                IVec3::new(0, -1, 0),
                IVec3::new(0, 0, -1),
            ),
            Transform::RotateClockwise240 => IMat3::from_cols(
                IVec3::new(0, 0, 1),
                IVec3::new(1, 0, 0),
                IVec3::new(0, 1, 0),
            ),
            Transform::RotateClockwise300 => IMat3::from_cols(
                IVec3::new(0, -1, 0),
                IVec3::new(0, 0, -1),
                IVec3::new(-1, 0, 0),
            ),
            Transform::ReflectQ => IMat3::from_cols(
                IVec3::new(1, 0, 0),
                IVec3::new(0, 0, 1),
                IVec3::new(0, 1, 0),
            ),
            Transform::ReflectR => IMat3::from_cols(
                IVec3::new(0, 0, 1),
                IVec3::new(0, 1, 0),
                IVec3::new(1, 0, 0),
            ),
            Transform::ReflectS => IMat3::from_cols(
                IVec3::new(0, 1, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(0, 0, 1),
            ),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TransformMatrix {
    value: IMat3,
}

impl TransformMatrix {
    pub fn apply(&self, coord: HexCoord) -> HexCoord {
        (self.value * coord.qrs()).try_into().unwrap()
    }

    pub const fn from_transform(transform: Transform) -> Self {
        Self {
            value: transform.into_imat3(),
        }
    }

    pub fn from_transforms<I: Iterator<Item = Transform>>(iter: I) -> Self {
        Self {
            value: iter
                .map(|transform| transform.into_imat3())
                .reduce(|acc, tmat| acc * tmat)
                .unwrap(),
        }
    }
}

impl From<Transform> for TransformMatrix {
    fn from(transform: Transform) -> Self {
        TransformMatrix::from_transform(transform)
    }
}

#[cfg(test)]
mod tests {
    use super::{HexCoord, Transform, TransformMatrix};
    use glam::IVec3;

    #[test]
    fn transform_apply() {
        let source = HexCoord::new(2, -2);
        assert_eq!(Transform::Identity.apply(source), HexCoord::new(2, -2));
        assert_eq!(
            Transform::RotateClockwise60.apply(source),
            HexCoord::new(2, 0)
        );
        assert_eq!(
            Transform::RotateClockwise120.apply(source),
            HexCoord::new(0, 2)
        );
        assert_eq!(
            Transform::RotateClockwise60.apply(Transform::RotateClockwise60.apply(source)),
            HexCoord::new(0, 2)
        );
    }

    #[test]
    fn transfrom_imat3() {
        let source = HexCoord::new(2, -2);
        assert_eq!(
            Transform::Identity.into_imat3() * source.qrs(),
            IVec3::new(2, -2, 0),
        );
        assert_eq!(
            Transform::RotateClockwise60.into_imat3() * source.qrs(),
            IVec3::new(2, 0, -2),
        );
        assert_eq!(
            Transform::RotateClockwise120.into_imat3() * source.qrs(),
            IVec3::new(0, 2, -2),
        );
        let rotate_twice =
            Transform::RotateClockwise60.into_imat3() * Transform::RotateClockwise60.into_imat3();
        assert_eq!(rotate_twice * source.qrs(), IVec3::new(0, 2, -2),);
    }

    #[test]
    fn transform_matrix() {
        let source = HexCoord::new(2, -2);
        let transform = TransformMatrix::from_transforms(
            [Transform::RotateClockwise60, Transform::RotateClockwise60].into_iter(),
        );
        assert_eq!(transform.apply(source), HexCoord::new(0, 2));
    }
}
