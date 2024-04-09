use ilattice::glam::UVec3;
use ilattice::prelude::Extent;
use ndcopy::fill3;
use ndshape::Shape;

pub use merge_strategy::*;

use crate::{
    bounds::assert_in_bounds, OrientedBlockFace, QuadBuffer, UnorientedQuad, Voxel, VoxelVisibility,
};

mod merge_strategy;

pub trait MergeVoxel: Voxel {
    type MergeValue: Eq;
    type MergeValueFacingNeighbour: Eq;

    /// The value used to determine if this voxel can join a given quad in the mesh. This value will be constant for all voxels
    /// in the same quad. Often this is some material identifier so that the same texture can be used for a full quad.
    fn merge_value(&self) -> Self::MergeValue;

    fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour;
}

/// Contains the output from the [`greedy_quads`] algorithm. The quads can be used to generate a mesh. See the methods on
/// [`OrientedBlockFace`] and [`UnorientedQuad`] for details.
///
/// This buffer can be reused between multiple calls of [`greedy_quads`] in order to avoid reallocations.
pub struct GreedyQuadsBuffer<V: Copy> {
    pub quads: QuadBuffer<V>,

    // A single array is used for the visited mask because it allows us to index by the same strides as the voxels array. It
    // also only requires a single allocation.
    visited: Vec<bool>,
}

impl<V: Copy> GreedyQuadsBuffer<V> {
    pub fn new(size: usize) -> Self {
        Self {
            quads: QuadBuffer::new(),
            visited: vec![false; size],
        }
    }

    pub fn reset(&mut self, size: usize) {
        self.quads.reset();

        if size != self.visited.len() {
            self.visited = vec![false; size];
        }
    }
}

/// The "Greedy Meshing" algorithm described by Mikola Lysenko in the [0fps
/// article](https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/).
///
/// All visible faces of voxels on the interior of `[min, max]` will be part of some [`UnorientedQuad`] returned via the
/// `output` buffer. A 3x3x3 kernel will be applied to each point on the interior, hence the extra padding required on the
/// boundary. `voxels` only needs to contain the set of points in `[min, max]`.
///
/// All quads created will have the same "merge value" as defined by the [`MergeVoxel`] trait. The quads can be post-processed
/// into meshes as the user sees fit.
pub fn greedy_quads<T: Copy, S>(
    voxels: &[T],
    voxels_shape: &S,
    min: [u32; 3],
    max: [u32; 3],
    faces: &[OrientedBlockFace; 6],
    output: &mut GreedyQuadsBuffer<T>,
) where
    T: MergeVoxel,
    S: Shape<3, Coord = u32>,
{
    greedy_quads_with_merge_strategy::<_, _, VoxelMerger<T>>(
        voxels,
        voxels_shape,
        min,
        max,
        faces,
        output,
    )
}

/// Run the greedy meshing algorithm with a custom quad merging strategy using the [`MergeStrategy`] trait.
pub fn greedy_quads_with_merge_strategy<T: Copy, S, Merger>(
    voxels: &[T],
    voxels_shape: &S,
    min: [u32; 3],
    max: [u32; 3],
    faces: &[OrientedBlockFace; 6],
    output: &mut GreedyQuadsBuffer<T>,
) where
    T: Voxel,
    S: Shape<3, Coord = u32>,
    Merger: MergeStrategy<Voxel = T>,
{
    assert_in_bounds(voxels, voxels_shape, min, max);

    let min = UVec3::from(min).as_ivec3();
    let max = UVec3::from(max).as_ivec3();
    let extent = Extent::from_min_and_max(min, max);

    output.reset(voxels.len());
    let GreedyQuadsBuffer {
        visited,
        quads: QuadBuffer { groups },
    } = output;

    let interior = extent.padded(-1); // Avoid accessing out of bounds with a 3x3x3 kernel.
    let interior =
        Extent::from_min_and_shape(interior.minimum.as_uvec3(), interior.shape.as_uvec3());

    for (group, face) in groups.iter_mut().zip(faces.iter()) {
        greedy_quads_for_face::<_, _, Merger>(voxels, voxels_shape, interior, face, visited, group);
    }
}

fn greedy_quads_for_face<T: Copy, S, Merger>(
    voxels: &[T],
    voxels_shape: &S,
    interior: Extent<UVec3>,
    face: &OrientedBlockFace,
    visited: &mut [bool],
    quads: &mut Vec<UnorientedQuad<T>>,
) where
    T: Voxel,
    S: Shape<3, Coord = u32>,
    Merger: MergeStrategy<Voxel = T>,
{
    visited.fill(false);

    let OrientedBlockFace {
        n_sign,
        permutation,
        n,
        u,
        v,
        ..
    } = face;

    let [n_axis, u_axis, v_axis] = permutation.axes();
    let i_n = n_axis.index();
    let i_u = u_axis.index();
    let i_v = v_axis.index();

    let interior_shape = interior.shape.to_array();
    let num_slices = interior_shape[i_n];
    let mut slice_shape = [0; 3];
    slice_shape[i_n] = 1;
    slice_shape[i_u] = interior_shape[i_u];
    slice_shape[i_v] = interior_shape[i_v];
    let mut slice_extent = Extent::from_min_and_shape(interior.minimum, UVec3::from(slice_shape));

    let n_stride = voxels_shape.linearize(n.to_array());
    let u_stride = voxels_shape.linearize(u.to_array());
    let v_stride = voxels_shape.linearize(v.to_array());
    let face_strides = FaceStrides {
        n_stride,
        u_stride,
        v_stride,
        // The offset to the voxel sharing this cube face.
        visibility_offset: if *n_sign > 0 {
            n_stride
        } else {
            0u32.wrapping_sub(n_stride)
        },
    };

    for _ in 0..num_slices {
        let slice_ub = slice_extent.least_upper_bound().to_array();
        let u_ub = slice_ub[i_u];
        let v_ub = slice_ub[i_v];

        for quad_min in slice_extent.iter3() {
            let quad_min_array = quad_min.to_array();
            let quad_min_index = voxels_shape.linearize(quad_min_array);
            let quad_min_voxel = unsafe { voxels.get_unchecked(quad_min_index as usize) };
            if unsafe {
                !face_needs_mesh(
                    quad_min_voxel,
                    quad_min_index,
                    face_strides.visibility_offset,
                    voxels,
                    visited,
                )
            } {
                continue;
            }
            // We have at least one face that needs a mesh. We'll try to expand that face into the biggest quad we can find.

            // These are the boundaries on quad width and height so it is contained in the slice.
            let max_width = u_ub - quad_min_array[i_u];
            let max_height = v_ub - quad_min_array[i_v];

            let (quad_width, quad_height) = unsafe {
                Merger::find_quad(
                    quad_min_index,
                    max_width,
                    max_height,
                    &face_strides,
                    voxels,
                    visited,
                )
            };
            debug_assert!(quad_width >= 1);
            debug_assert!(quad_width <= max_width);
            debug_assert!(quad_height >= 1);
            debug_assert!(quad_height <= max_height);

            // Mark the quad as visited.
            let mut quad_shape = [0; 3];
            quad_shape[i_n] = 1;
            quad_shape[i_u] = quad_width;
            quad_shape[i_v] = quad_height;
            fill3(quad_shape, true, visited, voxels_shape, quad_min_array);

            quads.push(UnorientedQuad {
                minimum: quad_min.to_array(),
                width: quad_width,
                height: quad_height,
                voxel: *quad_min_voxel,
            });
        }

        // Move to the next slice.
        slice_extent = slice_extent + *n;
    }
}

/// Returns true iff the given `voxel` face needs to be meshed. This means that we haven't already meshed it, it is non-empty,
/// and it's visible (not completely occluded by an adjacent voxel).
pub(crate) unsafe fn face_needs_mesh<T>(
    voxel: &T,
    voxel_stride: u32,
    visibility_offset: u32,
    voxels: &[T],
    visited: &[bool],
) -> bool
where
    T: Voxel,
{
    if voxel.get_visibility() == VoxelVisibility::Empty || visited[voxel_stride as usize] {
        return false;
    }

    let adjacent_voxel =
        voxels.get_unchecked(voxel_stride.wrapping_add(visibility_offset) as usize);

    // TODO: If the face lies between two transparent voxels, we choose not to mesh it. We might need to extend the IsOpaque
    // trait with different levels of transparency to support this.
    match adjacent_voxel.get_visibility() {
        VoxelVisibility::Empty => true,
        VoxelVisibility::Translucent => voxel.get_visibility() == VoxelVisibility::Opaque,
        VoxelVisibility::Opaque => false,
    }
}

#[cfg(test)]
mod tests {
    use ndshape::{ConstShape, ConstShape3u32};

    use crate::RIGHT_HANDED_Y_UP_CONFIG;

    use super::*;

    #[test]
    #[should_panic]
    fn panics_with_max_out_of_bounds_access() {
        let samples = [EMPTY; SampleShape::SIZE as usize];
        let mut buffer = GreedyQuadsBuffer::new(samples.len());
        greedy_quads(
            &samples,
            &SampleShape {},
            [0; 3],
            [34, 33, 33],
            &RIGHT_HANDED_Y_UP_CONFIG.faces,
            &mut buffer,
        );
    }

    #[test]
    #[should_panic]
    fn panics_with_min_out_of_bounds_access() {
        let samples = [EMPTY; SampleShape::SIZE as usize];
        let mut buffer = GreedyQuadsBuffer::new(samples.len());
        greedy_quads(
            &samples,
            &SampleShape {},
            [0, 34, 0],
            [33; 3],
            &RIGHT_HANDED_Y_UP_CONFIG.faces,
            &mut buffer,
        );
    }

    type SampleShape = ConstShape3u32<34, 34, 34>;

    /// Basic voxel type with one byte of texture layers
    #[derive(Default, Clone, Copy, Eq, PartialEq)]
    struct BoolVoxel(bool);

    const EMPTY: BoolVoxel = BoolVoxel(false);

    impl Voxel for BoolVoxel {
        fn get_visibility(&self) -> VoxelVisibility {
            if *self == EMPTY {
                VoxelVisibility::Empty
            } else {
                VoxelVisibility::Opaque
            }
        }
    }

    impl MergeVoxel for BoolVoxel {
        type MergeValue = Self;
        type MergeValueFacingNeighbour = bool;

        fn merge_value(&self) -> Self::MergeValue {
            *self
        }

        fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour {
            true
        }
    }
}
