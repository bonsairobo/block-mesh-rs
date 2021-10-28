use crate::{OrientedBlockFace, UnitQuadBuffer, UnorientedUnitQuad, Voxel};

use ilattice::glam::UVec3;
use ilattice::prelude::Extent;
use ndshape::Shape;

/// A fast and simple meshing algorithm that produces a single quad for every visible face of a block.
///
/// This is faster than [`greedy_quads`](crate::greedy_quads) but it produces many more quads.
pub fn visible_block_faces<T, S>(
    voxels: &[T],
    voxels_shape: &S,
    min: [u32; 3],
    max: [u32; 3],
    faces: &[OrientedBlockFace; 6],
    output: &mut UnitQuadBuffer,
) where
    T: Voxel,
    S: Shape<u32, 3>,
{
    assert!(
        voxels_shape.size() as usize <= voxels.len(),
        "voxel buffer size {:?} is less than the shape size {:?}; would cause access out-of-bounds",
        voxels.len(),
        voxels_shape.size()
    );

    let min = UVec3::from(min).as_ivec3();
    let max = UVec3::from(max).as_ivec3();
    let extent = Extent::from_min_and_max(min, max);
    let interior = extent.padded(-1); // Avoid accessing out of bounds with a 3x3x3 kernel.
    let interior =
        Extent::from_min_and_shape(interior.minimum.as_uvec3(), interior.shape.as_uvec3());

    let kernel_strides =
        faces.map(|face| voxels_shape.linearize(face.signed_normal().as_uvec3().to_array()));

    for p in interior.iter3() {
        let p_array = p.to_array();
        let p_index = voxels_shape.linearize(p_array);
        let p_voxel = &voxels[p_index as usize];

        if p_voxel.is_empty() {
            continue;
        }

        for (face_index, face_stride) in kernel_strides.into_iter().enumerate() {
            let neighbor_index = p_index.wrapping_add(face_stride);
            let neighbor_voxel = &voxels[neighbor_index as usize];

            // TODO: If the face lies between two transparent voxels, we choose not to mesh it. We might need to extend the
            // IsOpaque trait with different levels of transparency to support this.
            let face_needs_mesh =
                neighbor_voxel.is_empty() || (!neighbor_voxel.is_opaque() && p_voxel.is_opaque());

            if face_needs_mesh {
                output.groups[face_index].push(UnorientedUnitQuad { minimum: p_array });
            }
        }
    }
}
