# block-mesh

[![Crates.io](https://img.shields.io/crates/v/block-mesh.svg)](https://crates.io/crates/block-mesh)
[![Docs.rs](https://docs.rs/block-mesh/badge.svg)](https://docs.rs/block-mesh)

Fast algorithms for generating voxel block meshes.

![Mesh Examples](https://raw.githubusercontent.com/bonsairobo/block-mesh-rs/main/examples-crate/render/mesh_examples.png)

Two algorithms are included:
- [`visible_block_faces`](crate::visible_block_faces): very fast but suboptimal meshes
- [`greedy_quads`](crate::greedy_quads): not quite as fast, but far fewer triangles are generated

Benchmarks show that [`visible_block_faces`](crate::visible_block_faces) generates about 40 million quads per second on a
single core of a 2.5 GHz Intel Core i7. Assuming spherical input data, [`greedy_quads`](crate::greedy_quads) can generate a
more optimal version of the same mesh with 1/3 of the quads, but it takes about 3 times longer. To run the benchmarks
yourself, `cd bench/ && cargo bench`.

## Example Code

```rust
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};

#[derive(Clone, Copy, Eq, PartialEq)]
struct BoolVoxel(bool);

const EMPTY: BoolVoxel = BoolVoxel(false);
const FULL: BoolVoxel = BoolVoxel(true);

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

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

// A 16^3 chunk with 1-voxel boundary padding.
type ChunkShape = ConstShape3u32<18, 18, 18>;

// This chunk will cover just a single octant of a sphere SDF (radius 15).
let mut voxels = [EMPTY; ChunkShape::SIZE as usize];
for i in 0..ChunkShape::SIZE {
    let [x, y, z] = ChunkShape::delinearize(i);
    voxels[i as usize] = if ((x * x + y * y + z * z) as f32).sqrt() < 15.0 {
        FULL
    } else {
        EMPTY
    };
}

let mut buffer = GreedyQuadsBuffer::new(voxels.len());
greedy_quads(
    &voxels,
    &ChunkShape {},
    [0; 3],
    [17; 3],
    &RIGHT_HANDED_Y_UP_CONFIG.faces,
    &mut buffer
);

// Some quads were generated.
assert!(buffer.quads.num_quads() > 0);
```

License: MIT OR Apache-2.0
