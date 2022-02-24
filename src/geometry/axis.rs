use ilattice::glam::{IVec3, UVec3};

/// Either the X, Y, or Z axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    /// The index for a point's component on this axis.
    #[inline]
    pub fn index(&self) -> usize {
        *self as usize
    }

    #[inline]
    pub const fn get_unit_vector(&self) -> UVec3 {
        match self {
            Axis::X => UVec3::X,
            Axis::Y => UVec3::Y,
            Axis::Z => UVec3::Z,
        }
    }
}

/// One of the six possible `{N, U, V}` --> `{X, Y, Z}` mappings.
///
/// This can be combined with a `-1` or `+1` sign for the **N**ormal axis to
/// make an [`OrientedBlockFace`][crate::OrientedBlockFace].
///
/// See the [`geometry` module documentation][crate::geometry] for more
/// information on `{N, U, V}` space.
///
/// # Even and Odd
///
/// Even permutations:
/// - [AxisPermutation::Xyz]
/// - [AxisPermutation::Zxy]
/// - [AxisPermutation::Yzx]
///
/// Odd permutations:
/// - [AxisPermutation::Zyx]
/// - [AxisPermutation::Xzy]
/// - [AxisPermutation::Yxz]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxisPermutation {
    // Even permutations
    Xyz,
    Zxy,
    Yzx,
    // Odd permutations
    Zyx,
    Xzy,
    Yxz,
}

impl AxisPermutation {
    #[inline]
    pub const fn even_with_normal_axis(axis: Axis) -> Self {
        match axis {
            Axis::X => AxisPermutation::Xyz,
            Axis::Y => AxisPermutation::Yzx,
            Axis::Z => AxisPermutation::Zxy,
        }
    }

    #[inline]
    pub const fn odd_with_normal_axis(axis: Axis) -> Self {
        match axis {
            Axis::X => AxisPermutation::Xzy,
            Axis::Y => AxisPermutation::Yxz,
            Axis::Z => AxisPermutation::Zyx,
        }
    }

    #[inline]
    pub const fn sign(&self) -> i32 {
        match self {
            AxisPermutation::Xyz => 1,
            AxisPermutation::Zxy => 1,
            AxisPermutation::Yzx => 1,
            AxisPermutation::Zyx => -1,
            AxisPermutation::Xzy => -1,
            AxisPermutation::Yxz => -1,
        }
    }

    /// Returns the [`Axes`] in the order specified by the permutation.
    ///
    /// # Example
    ///
    /// ```
    /// # use block_mesh::*;
    /// let xyz = AxisPermutation::Xyz;
    /// assert_eq!(xyz.axes(), [Axis::X, Axis::Y, Axis::Z]);
    /// ```
    ///
    /// [`Axes`]: Axis
    #[inline]
    pub const fn axes(&self) -> [Axis; 3] {
        match self {
            AxisPermutation::Xyz => [Axis::X, Axis::Y, Axis::Z],
            AxisPermutation::Zxy => [Axis::Z, Axis::X, Axis::Y],
            AxisPermutation::Yzx => [Axis::Y, Axis::Z, Axis::X],
            AxisPermutation::Zyx => [Axis::Z, Axis::Y, Axis::X],
            AxisPermutation::Xzy => [Axis::X, Axis::Z, Axis::Y],
            AxisPermutation::Yxz => [Axis::Y, Axis::X, Axis::Z],
        }
    }
}

/// Either the -X, +X, -Y, +Y, -Z, or +Z axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SignedAxis {
    NegX = 0,
    PosX = 1,
    NegY = 2,
    PosY = 3,
    NegZ = 4,
    PosZ = 5,
}

impl SignedAxis {
    #[inline]
    pub fn new(sign: i32, axis: Axis) -> Self {
        assert!(sign != 0);

        match (sign > 0, axis) {
            (false, Axis::X) => Self::NegX,
            (false, Axis::Y) => Self::NegY,
            (false, Axis::Z) => Self::NegZ,
            (true, Axis::X) => Self::PosX,
            (true, Axis::Y) => Self::PosY,
            (true, Axis::Z) => Self::PosZ,
        }
    }

    #[inline]
    pub fn unsigned_axis(&self) -> Axis {
        match self {
            Self::NegX => Axis::X,
            Self::NegY => Axis::Y,
            Self::NegZ => Axis::Z,
            Self::PosX => Axis::X,
            Self::PosY => Axis::Y,
            Self::PosZ => Axis::Z,
        }
    }

    #[inline]
    pub fn signum(&self) -> i32 {
        match self {
            Self::NegX => -1,
            Self::NegY => -1,
            Self::NegZ => -1,
            Self::PosX => 1,
            Self::PosY => 1,
            Self::PosZ => 1,
        }
    }

    #[inline]
    pub fn get_unit_vector(&self) -> IVec3 {
        match self {
            Self::NegX => -IVec3::X,
            Self::NegY => -IVec3::Y,
            Self::NegZ => -IVec3::Z,
            Self::PosX => IVec3::X,
            Self::PosY => IVec3::Y,
            Self::PosZ => IVec3::Z,
        }
    }

    #[inline]
    pub fn from_vector(v: IVec3) -> Option<Self> {
        match v.to_array() {
            [x, 0, 0] => Some(SignedAxis::new(x, Axis::X)),
            [0, y, 0] => Some(SignedAxis::new(y, Axis::Y)),
            [0, 0, z] => Some(SignedAxis::new(z, Axis::Z)),
            _ => None,
        }
    }
}
