use crate::{UnorientedQuad, UnorientedUnitQuad};

#[derive(Default)]
pub struct QuadBuffer {
    /// A group of quads for each block face. We rely on [`OrientedBlockFace`]
    /// metadata to interpret them.
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

#[derive(Default)]
pub struct UnitQuadBuffer {
    /// A group of quads for each block face. We rely on [`OrientedBlockFace`]
    /// metadata to interpret them.
    ///
    /// When using these values for materials and lighting, you can access them
    /// using either the quad's minimum voxel coordinates or the vertex
    /// coordinates given by [`OrientedBlockFace::quad_corners`].
    pub groups: [Vec<UnorientedUnitQuad>; 6],
}

impl UnitQuadBuffer {
    pub fn new() -> Self {
        const EMPTY: Vec<UnorientedUnitQuad> = Vec::new();
        Self { groups: [EMPTY; 6] }
    }

    /// Clears the buffer.
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
