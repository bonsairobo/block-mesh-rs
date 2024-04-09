use crate::{UnorientedQuad, UnorientedUnitQuad};

#[derive(Default)]
pub struct QuadBuffer<V: Copy> {
    /// A group of quads for each block face. We rely on [`OrientedBlockFace`]
    /// metadata to interpret them.
    pub groups: [Vec<UnorientedQuad<V>>; 6],
}

impl<V: Copy> QuadBuffer<V> {
    pub fn new() -> Self {
        Self {
            groups: [
                Vec::<UnorientedQuad<V>>::new(),
                Vec::<UnorientedQuad<V>>::new(),
                Vec::<UnorientedQuad<V>>::new(),
                Vec::<UnorientedQuad<V>>::new(),
                Vec::<UnorientedQuad<V>>::new(),
                Vec::<UnorientedQuad<V>>::new(),
            ],
        }
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
pub struct UnitQuadBuffer<V> {
    /// A group of quads for each block face. We rely on [`OrientedBlockFace`]
    /// metadata to interpret them.
    ///
    /// When using these values for materials and lighting, you can access them
    /// using either the quad's minimum voxel coordinates or the vertex
    /// coordinates given by [`OrientedBlockFace::quad_corners`].
    pub groups: [Vec<UnorientedUnitQuad<V>>; 6],
}

impl<V: Copy> UnitQuadBuffer<V> {
    pub fn new() -> Self {
        Self {
            groups: [
                Vec::<UnorientedUnitQuad<V>>::new(),
                Vec::<UnorientedUnitQuad<V>>::new(),
                Vec::<UnorientedUnitQuad<V>>::new(),
                Vec::<UnorientedUnitQuad<V>>::new(),
                Vec::<UnorientedUnitQuad<V>>::new(),
                Vec::<UnorientedUnitQuad<V>>::new(),
            ],
        }
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
