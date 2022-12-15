use crate::greedy::face_needs_mesh;
use crate::Voxel;

use super::MergeVoxel;

// TODO: implement a MergeStrategy for voxels with an ambient occlusion value at each vertex

/// A strategy for merging cube faces into quads.
pub trait MergeStrategy {
    type Voxel;

    /// Return the width and height of the quad that should be constructed.
    ///
    /// `min_index`: The linear index for the minimum voxel in this quad.
    ///
    /// `max_width`: The maximum possible width for the quad to be constructed.
    ///
    /// `max_height`: The maximum possible height for the quad to be constructed.
    ///
    /// `face_strides`: Strides to help with indexing in the necessary directions for this cube face.
    ///
    /// `voxels`: The entire array of voxel data.
    ///
    /// `visited`: The bitmask of which voxels have already been meshed. A quad's extent will be marked as visited (`true`)
    ///            after `find_quad` returns.
    ///
    /// # Safety
    ///
    /// Some implementations may use unchecked indexing of `voxels` for performance. If this trait is not invoked with correct
    /// arguments, access out of bounds may cause undefined behavior.
    unsafe fn find_quad(
        min_index: u32,
        max_width: u32,
        max_height: u32,
        face_strides: &FaceStrides,
        voxels: &[Self::Voxel],
        visited: &[bool],
    ) -> (u32, u32)
    where
        Self::Voxel: Voxel;
}

pub struct FaceStrides {
    pub n_stride: u32,
    pub u_stride: u32,
    pub v_stride: u32,
    pub visibility_offset: u32,
}

pub struct VoxelMerger<T> {
    marker: std::marker::PhantomData<T>,
}

impl<T> MergeStrategy for VoxelMerger<T>
where
    T: MergeVoxel,
{
    type Voxel = T;

    unsafe fn find_quad(
        min_index: u32,
        max_width: u32,
        max_height: u32,
        face_strides: &FaceStrides,
        voxels: &[T],
        visited: &[bool],
    ) -> (u32, u32) {
        // Greedily search for the biggest visible quad where all merge values are the same.
        let quad_value = voxels.get_unchecked(min_index as usize).merge_value();
        let quad_neighbour_value = voxels
            .get_unchecked(min_index.wrapping_add(face_strides.visibility_offset) as usize)
            .merge_value_facing_neighbour();

        // Start by finding the widest quad in the U direction.
        let mut row_start_stride = min_index;
        let quad_width = Self::get_row_width(
            voxels,
            visited,
            &quad_value,
            &quad_neighbour_value,
            face_strides.visibility_offset,
            row_start_stride,
            face_strides.u_stride,
            max_width,
        );

        // Now see how tall we can make the quad in the V direction without changing the width.
        row_start_stride += face_strides.v_stride;
        let mut quad_height = 1;
        while quad_height < max_height {
            let row_width = Self::get_row_width(
                voxels,
                visited,
                &quad_value,
                &quad_neighbour_value,
                face_strides.visibility_offset,
                row_start_stride,
                face_strides.u_stride,
                quad_width,
            );
            if row_width < quad_width {
                break;
            }
            quad_height += 1;
            row_start_stride = row_start_stride.wrapping_add(face_strides.v_stride);
        }

        (quad_width, quad_height)
    }
}

impl<T> VoxelMerger<T> {
    unsafe fn get_row_width(
        voxels: &[T],
        visited: &[bool],
        quad_merge_voxel_value: &T::MergeValue,
        quad_merge_voxel_value_facing_neighbour: &T::MergeValueFacingNeighbour,
        visibility_offset: u32,
        start_stride: u32,
        delta_stride: u32,
        max_width: u32,
    ) -> u32
    where
        T: MergeVoxel,
    {
        let mut quad_width = 0;
        let mut row_stride = start_stride;
        while quad_width < max_width {
            let voxel = voxels.get_unchecked(row_stride as usize);
            let neighbour =
                voxels.get_unchecked(row_stride.wrapping_add(visibility_offset) as usize);

            if !face_needs_mesh(voxel, row_stride, visibility_offset, voxels, visited) {
                break;
            }

            if !voxel.merge_value().eq(quad_merge_voxel_value)
                || !neighbour
                    .merge_value_facing_neighbour()
                    .eq(quad_merge_voxel_value_facing_neighbour)
            {
                // Voxel needs to be non-empty and match the quad merge value.
                break;
            }

            quad_width += 1;
            row_stride += delta_stride;
        }

        quad_width
    }
}
