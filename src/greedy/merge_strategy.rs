use std::collections::HashMap;

use ndshape::Shape;

use crate::greedy::face_needs_mesh;
use crate::{Voxel, VoxelVisibility};

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
        voxels_shape: &dyn Shape<3, Coord = u32>,
        aos: &mut HashMap<(u32, u8), [u8; 4]>,
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
        voxels_shape: &dyn Shape<3, Coord = u32>,
        aos: &mut HashMap<(u32, u8), [u8; 4]>,
    ) -> (u32, u32) {
        // Greedily search for the biggest visible quad where all merge values are the same.
        let quad_value = voxels.get_unchecked(min_index as usize).merge_value();
        let quad_neighbour_value = voxels.get_unchecked((min_index + face_strides.visibility_offset) as usize).merge_value_facing_neighbour();

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
            voxels_shape,
            aos,
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
                voxels_shape,
                aos,
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
    fn get_face_index(visibility_offset: u32, start_stride: u32, voxels_shape: &dyn Shape<3, Coord = u32>) -> u8 {
        let [x, y, z] = voxels_shape.delinearize(start_stride);
        let [nx, ny, nz] = voxels_shape.delinearize(start_stride + visibility_offset);

        if nx < x {
            0
        } else if ny < y {
            1
        } else if nz < z {
            2
        } else if nx > x {
            3
        } else if ny > y {
            4
        } else if nz > z {
            5
        } else {
            0
        }
    }

    //[-1, 0, 0], // left
    //[0, -1, 0], // bottom
    //[0, 0, -1], // back
    //[1, 0, 0],  // right
    //[0, 1, 0],  // top
    //[0, 0, 1],  // front

    fn calculate_ao(voxels: &[T], visibility_offset: u32, stride: u32, current_face: u8, voxels_shape: &dyn Shape<3, Coord = u32>) -> [u8; 4]
    where
        T: MergeVoxel,
    {
        let [x, y, z] = voxels_shape.delinearize(stride + visibility_offset);

        let neighbours: [&T; 8];

        if current_face == 0 || current_face == 3 {
            // left or right
            neighbours = [
                &voxels[voxels_shape.linearize([x, y, z + 1]) as usize],
                &voxels[voxels_shape.linearize([x, y - 1, z + 1]) as usize],
                &voxels[voxels_shape.linearize([x, y - 1, z]) as usize],
                &voxels[voxels_shape.linearize([x, y - 1, z - 1]) as usize],
                &voxels[voxels_shape.linearize([x, y, z - 1]) as usize],
                &voxels[voxels_shape.linearize([x, y + 1, z - 1]) as usize],
                &voxels[voxels_shape.linearize([x, y + 1, z]) as usize],
                &voxels[voxels_shape.linearize([x, y + 1, z + 1]) as usize],
            ];
        } else if current_face == 1 || current_face == 4 {
            // bottom or top
            neighbours = [
                &voxels[voxels_shape.linearize([x, y, z + 1]) as usize],
                &voxels[voxels_shape.linearize([x - 1, y, z + 1]) as usize],
                &voxels[voxels_shape.linearize([x - 1, y, z]) as usize],
                &voxels[voxels_shape.linearize([x - 1, y, z - 1]) as usize],
                &voxels[voxels_shape.linearize([x, y, z - 1]) as usize],
                &voxels[voxels_shape.linearize([x + 1, y, z - 1]) as usize],
                &voxels[voxels_shape.linearize([x + 1, y, z]) as usize],
                &voxels[voxels_shape.linearize([x + 1, y, z + 1]) as usize],
            ];
        } else {
            // back or front
            neighbours = [
                &voxels[voxels_shape.linearize([x + 1, y, z]) as usize],
                &voxels[voxels_shape.linearize([x + 1, y - 1, z]) as usize],
                &voxels[voxels_shape.linearize([x, y - 1, z]) as usize],
                &voxels[voxels_shape.linearize([x - 1, y - 1, z]) as usize],
                &voxels[voxels_shape.linearize([x - 1, y, z]) as usize],
                &voxels[voxels_shape.linearize([x - 1, y + 1, z]) as usize],
                &voxels[voxels_shape.linearize([x, y + 1, z]) as usize],
                &voxels[voxels_shape.linearize([x + 1, y + 1, z]) as usize],
            ];
        }

        let mut ao = [0; 4];
        if neighbours[0].get_visibility() == VoxelVisibility::Opaque && neighbours[2].get_visibility() == VoxelVisibility::Opaque {
            ao[1] = 0;
        } else if neighbours[1].get_visibility() == VoxelVisibility::Opaque && (neighbours[0].get_visibility() == VoxelVisibility::Opaque || neighbours[2].get_visibility() == VoxelVisibility::Opaque) {
            ao[1] = 1;
        } else if neighbours[0].get_visibility() == VoxelVisibility::Opaque || neighbours[1].get_visibility() == VoxelVisibility::Opaque || neighbours[2].get_visibility() == VoxelVisibility::Opaque {
            ao[1] = 2;
        } else {
            ao[1] = 3;
        }
        if neighbours[2].get_visibility() == VoxelVisibility::Opaque && neighbours[4].get_visibility() == VoxelVisibility::Opaque {
            ao[0] = 0;
        } else if neighbours[3].get_visibility() == VoxelVisibility::Opaque && (neighbours[2].get_visibility() == VoxelVisibility::Opaque || neighbours[4].get_visibility() == VoxelVisibility::Opaque) {
            ao[0] = 1;
        } else if neighbours[2].get_visibility() == VoxelVisibility::Opaque || neighbours[3].get_visibility() == VoxelVisibility::Opaque || neighbours[4].get_visibility() == VoxelVisibility::Opaque {
            ao[0] = 2;
        } else {
            ao[0] = 3;
        }
        if neighbours[4].get_visibility() == VoxelVisibility::Opaque && neighbours[6].get_visibility() == VoxelVisibility::Opaque {
            ao[2] = 0;
        } else if neighbours[5].get_visibility() == VoxelVisibility::Opaque && (neighbours[4].get_visibility() == VoxelVisibility::Opaque || neighbours[6].get_visibility() == VoxelVisibility::Opaque) {
            ao[2] = 1;
        } else if neighbours[4].get_visibility() == VoxelVisibility::Opaque || neighbours[5].get_visibility() == VoxelVisibility::Opaque || neighbours[6].get_visibility() == VoxelVisibility::Opaque {
            ao[2] = 2;
        } else {
            ao[2] = 3;
        }
        if neighbours[6].get_visibility() == VoxelVisibility::Opaque && neighbours[0].get_visibility() == VoxelVisibility::Opaque {
            ao[3] = 0;
        } else if neighbours[7].get_visibility() == VoxelVisibility::Opaque && (neighbours[6].get_visibility() == VoxelVisibility::Opaque || neighbours[0].get_visibility() == VoxelVisibility::Opaque) {
            ao[3] = 1;
        } else if neighbours[6].get_visibility() == VoxelVisibility::Opaque || neighbours[7].get_visibility() == VoxelVisibility::Opaque || neighbours[0].get_visibility() == VoxelVisibility::Opaque {
            ao[3] = 2;
        } else {
            ao[3] = 3;
        }

        ao
    }

    unsafe fn get_row_width(
        voxels: &[T],
        visited: &[bool],
        quad_merge_voxel_value: &T::MergeValue,
        quad_merge_voxel_value_facing_neighbour: &T::MergeValueFacingNeighbour,
        visibility_offset: u32,
        start_stride: u32,
        delta_stride: u32,
        max_width: u32,
        voxels_shape: &dyn Shape<3, Coord = u32>,
        aos: &mut HashMap<(u32, u8), [u8; 4]>,
    ) -> u32
    where
        T: MergeVoxel,
    {
        let mut quad_width = 0;
        let mut row_stride = start_stride;
        let current_face = Self::get_face_index(visibility_offset, start_stride, voxels_shape);
        while quad_width < max_width {
            let voxel = voxels.get_unchecked(row_stride as usize);
            let neighbour = voxels.get_unchecked((row_stride + visibility_offset) as usize);

            if !face_needs_mesh(voxel, row_stride, visibility_offset, voxels, visited) {
                break;
            }

            if !aos.contains_key(&(row_stride, current_face)) {
                let ao = Self::calculate_ao(voxels, visibility_offset, row_stride, current_face, voxels_shape);
                aos.insert((row_stride, current_face), ao);
            }
            if !aos.contains_key(&(start_stride, current_face)) {
                let ao = Self::calculate_ao(voxels, visibility_offset, start_stride, current_face, voxels_shape);
                aos.insert((start_stride, current_face), ao);
            }

            if !voxel.merge_value().eq(quad_merge_voxel_value)
                || !neighbour.merge_value_facing_neighbour().eq(quad_merge_voxel_value_facing_neighbour)
                || !aos.get(&(row_stride, current_face)).unwrap().eq(aos.get(&(start_stride, current_face)).unwrap())
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
