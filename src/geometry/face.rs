use crate::{Axis, AxisPermutation, SignedAxis, UnorientedQuad};

use ilattice::glam::{IVec3, UVec3};

/// Metadata that's used to aid in the geometric calculations for one of the 6 possible cube faces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrientedBlockFace {
    /// Determines the orientation of the plane.
    pub(crate) n_sign: i32,

    /// Determines the {N, U, V} <--> {X, Y, Z} relation.
    pub(crate) permutation: AxisPermutation,

    /// First in the `permutation` of +X, +Y, and +Z.
    pub(crate) n: UVec3,
    /// Second in the `permutation` of +X, +Y, and +Z.
    pub(crate) u: UVec3,
    /// Third in the `permutation` of +X, +Y, and +Z.
    pub(crate) v: UVec3,
}

impl OrientedBlockFace {
    pub const fn new(n_sign: i32, permutation: AxisPermutation) -> Self {
        let [n_axis, u_axis, v_axis] = permutation.axes();

        Self {
            n_sign,
            permutation,
            n: n_axis.get_unit_vector(),
            u: u_axis.get_unit_vector(),
            v: v_axis.get_unit_vector(),
        }
    }

    /// A cube face, using axes with an even permutation.
    pub fn canonical(normal: SignedAxis) -> Self {
        Self::new(
            normal.signum(),
            AxisPermutation::even_with_normal_axis(normal.unsigned_axis()),
        )
    }
    
    #[inline]
    pub fn n_sign(&self) -> i32 {
        self.n_sign
    }
    
    #[inline]
    pub fn permutation(&self) -> AxisPermutation {
        self.permutation
    }

    #[inline]
    pub fn signed_normal(&self) -> IVec3 {
        self.n.as_ivec3() * self.n_sign
    }

    /// Returns the 4 corners of the quad in this order:
    ///
    /// ```text
    ///         2 ----> 3
    ///           ^
    ///     ^       \
    ///     |         \
    ///  +V |   0 ----> 1
    ///     |
    ///      -------->
    ///        +U
    ///
    /// (+N pointing out of the screen)
    /// ```
    ///
    /// Note that this is natural when UV coordinates have (0,0) at the bottom
    /// left, but when (0,0) is at the top left, V must be flipped.
    #[inline]
    pub fn quad_corners(&self, quad: &UnorientedQuad) -> [UVec3; 4] {
        let w_vec = self.u * quad.width;
        let h_vec = self.v * quad.height;

        let minu_minv = if self.n_sign > 0 {
            UVec3::from(quad.minimum) + self.n
        } else {
            UVec3::from(quad.minimum)
        };
        let maxu_minv = minu_minv + w_vec;
        let minu_maxv = minu_minv + h_vec;
        let maxu_maxv = minu_minv + w_vec + h_vec;

        [minu_minv, maxu_minv, minu_maxv, maxu_maxv]
    }

    #[inline]
    pub fn quad_mesh_positions(&self, quad: &UnorientedQuad, voxel_size: f32) -> [[f32; 3]; 4] {
        self.quad_corners(quad)
            .map(|c| (voxel_size * c.as_vec3()).to_array())
    }

    #[inline]
    pub fn quad_mesh_normals(&self) -> [[f32; 3]; 4] {
        [self.signed_normal().as_vec3().to_array(); 4]
    }

    /// Returns the 6 vertex indices for the quad in order to make two triangles
    /// in a mesh. Winding order depends on both the sign of the surface normal
    /// and the permutation of the UVs.
    ///
    /// Front faces will be wound counterclockwise, and back faces clockwise, as
    /// per convention.
    #[inline]
    pub fn quad_mesh_indices(&self, start: u32) -> [u32; 6] {
        quad_indices(start, self.n_sign * self.permutation.sign() > 0)
    }

    /// Returns the UV coordinates of the 4 corners of the quad. Returns
    /// vertices in the same order as [`OrientedBlockFace::quad_corners`].
    ///
    /// `u_flip_face` should correspond to the field on
    /// [`QuadCoordinateConfig`](crate::QuadCoordinateConfig). See the docs
    /// there for more info.
    ///
    /// This is just one way of assigning UVs to voxel quads. It assumes that
    /// each material has a single tile texture with wrapping coordinates, and
    /// each voxel face should show the entire texture. It also assumes a
    /// particular orientation for the texture. This should be sufficient for
    /// minecraft-style meshing.
    ///
    /// If you need to use a texture atlas, you must calculate your own
    /// coordinates from the `Quad`.
    #[inline]
    pub fn tex_coords(
        &self,
        u_flip_face: Axis,
        flip_v: bool,
        quad: &UnorientedQuad,
    ) -> [[f32; 2]; 4] {
        let face_normal_axis = self.permutation.axes()[0];
        let flip_u = if self.n_sign < 0 {
            u_flip_face != face_normal_axis
        } else {
            u_flip_face == face_normal_axis
        };

        match (flip_u, flip_v) {
            (false, false) => [
                [0.0, 0.0],
                [quad.width as f32, 0.0],
                [0.0, quad.height as f32],
                [quad.width as f32, quad.height as f32],
            ],
            (true, false) => [
                [quad.width as f32, 0.0],
                [0.0, 0.0],
                [quad.width as f32, quad.height as f32],
                [0.0, quad.height as f32],
            ],
            (false, true) => [
                [0.0, quad.height as f32],
                [quad.width as f32, quad.height as f32],
                [0.0, 0.0],
                [quad.width as f32, 0.0],
            ],
            (true, true) => [
                [quad.width as f32, quad.height as f32],
                [0.0, quad.height as f32],
                [quad.width as f32, 0.0],
                [0.0, 0.0],
            ],
        }
    }
}

/// Returns the vertex indices for a single quad (two triangles). The triangles
/// may have either clockwise or counter-clockwise winding. `start` is the first
/// index.
fn quad_indices(start: u32, counter_clockwise: bool) -> [u32; 6] {
    if counter_clockwise {
        [start, start + 1, start + 2, start + 1, start + 3, start + 2]
    } else {
        [start, start + 2, start + 1, start + 1, start + 2, start + 3]
    }
}
