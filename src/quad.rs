use crate::{Axis, AxisPermutation, OrientedBlockFace};

/// A configuration of XYZ --> NUV axis mappings and orientations of the cube faces for a given coordinate system.
#[derive(Clone)]
pub struct QuadCoordinateConfig {
    pub faces: [OrientedBlockFace; 6],
    /// For a given coordinate system, one of the two axes that isn't UP must be flipped in the U texel coordinate direction to
    /// avoid incorrect texture mirroring. For example, in a right-handed coordinate system with +Y pointing up, you should set
    /// `u_flip_face` to [`Axis::X`], because those faces need their U coordinates to be flipped relative to the other faces.
    pub u_flip_face: Axis,
}

pub const RIGHT_HANDED_Y_UP_CONFIG: QuadCoordinateConfig = QuadCoordinateConfig {
    // Y is always in the V direction when it's not the normal. When Y is the normal, right-handedness determines that
    // we must use Yzx permutations.
    faces: [
        OrientedBlockFace::new(-1, AxisPermutation::Xzy),
        OrientedBlockFace::new(-1, AxisPermutation::Yzx),
        OrientedBlockFace::new(-1, AxisPermutation::Zxy),
        OrientedBlockFace::new(1, AxisPermutation::Xzy),
        OrientedBlockFace::new(1, AxisPermutation::Yzx),
        OrientedBlockFace::new(1, AxisPermutation::Zxy),
    ],
    u_flip_face: Axis::X,
};

#[derive(Default)]
pub struct QuadBuffer {
    /// A group of quads for each block face. We rely on [`OrientedBlockFace`] metadata to interpret them.
    ///
    /// When using these values for materials and lighting, you can access them using either the quad's minimum voxel
    /// coordinates or the vertex coordinates given by `OrientedBlockFace::quad_corners`.
    pub groups: [Vec<UnorientedQuad>; 6],
}

impl QuadBuffer {
    pub fn new() -> Self {
        const EMPTY: Vec<UnorientedQuad> = Vec::new();
        Self { groups: [EMPTY; 6] }
    }

    pub fn reset(&mut self) {
        for group in self.groups.iter_mut() {
            group.clear();
        }
    }

    /// Returns the total count of quads across all groups.
    pub fn num_quads(&self) -> usize {
        let mut sum = 0;
        for group in self.groups.iter() {
            sum += group.len();
        }
        sum
    }
}

/// The minimum voxel and size of a quad, without an orientation. To get the actual corners of the quad, combine with an
/// [`OrientedBlockFace`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnorientedQuad {
    /// The minimum voxel in the quad.
    pub minimum: [u32; 3],
    /// Width of the quad.
    pub width: u32,
    /// Height of the quad.
    pub height: u32,
}

impl From<UnorientedUnitQuad> for UnorientedQuad {
    #[inline]
    fn from(unit: UnorientedUnitQuad) -> Self {
        Self {
            minimum: unit.minimum,
            width: 1,
            height: 1,
        }
    }
}

#[derive(Default)]
pub struct UnitQuadBuffer {
    /// A group of quads for each block face. We rely on [`OrientedBlockFace`] metadata to interpret them.
    ///
    /// When using these values for materials and lighting, you can access them using either the quad's minimum voxel
    /// coordinates or the vertex coordinates given by `OrientedBlockFace::quad_corners`.
    pub groups: [Vec<UnorientedUnitQuad>; 6],
}

impl UnitQuadBuffer {
    pub fn new() -> Self {
        const EMPTY: Vec<UnorientedUnitQuad> = Vec::new();
        Self { groups: [EMPTY; 6] }
    }

    pub fn reset(&mut self) {
        for group in self.groups.iter_mut() {
            group.clear();
        }
    }

    /// Returns the total count of quads across all groups.
    pub fn num_quads(&self) -> usize {
        let mut sum = 0;
        for group in self.groups.iter() {
            sum += group.len();
        }
        sum
    }
}

/// A quad covering a single voxel (just a single block face), without an orientation. To get the actual corners of the quad,
/// combine with an [`OrientedBlockFace`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnorientedUnitQuad {
    /// The minimum voxel in the quad.
    pub minimum: [u32; 3],
}
