use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{
    greedy_quads, visible_block_faces, GreedyQuadsBuffer, MergeVoxel, UnitQuadBuffer, Voxel,
    VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

type SampleShape = ConstShape3u32<18, 18, 18>;

fn bench_empty_space_greedy(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_empty_space_greedy");
    let samples = [EMPTY; SampleShape::SIZE as usize];

    // Do a single run first to allocate the buffer to the right size.
    let mut buffer = GreedyQuadsBuffer::new(samples.len());
    greedy_quads(
        &samples,
        &SampleShape {},
        [0; 3],
        [17; 3],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );

    group.bench_with_input(
        BenchmarkId::from_parameter(format!("quads={}", buffer.quads.num_quads())),
        &(),
        |b, _| {
            b.iter(|| {
                greedy_quads(
                    &samples,
                    &SampleShape {},
                    [0; 3],
                    [17; 3],
                    &RIGHT_HANDED_Y_UP_CONFIG.faces,
                    &mut buffer,
                )
            });
        },
    );
    group.finish();
}

fn bench_sphere_greedy(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_sphere_greedy");
    let mut samples = [EMPTY; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(16, SampleShape::delinearize(i));
        samples[i as usize] = sphere_voxel(p);
    }

    // Do a single run first to allocate the buffer to the right size.
    let mut buffer = GreedyQuadsBuffer::new(samples.len());
    greedy_quads(
        &samples,
        &SampleShape {},
        [0; 3],
        [17; 3],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );

    group.bench_with_input(
        BenchmarkId::from_parameter(format!("quads={}", buffer.quads.num_quads())),
        &(),
        |b, _| {
            b.iter(|| {
                greedy_quads(
                    &samples,
                    &SampleShape {},
                    [0; 3],
                    [17; 3],
                    &RIGHT_HANDED_Y_UP_CONFIG.faces,
                    &mut buffer,
                )
            });
        },
    );
    group.finish();
}

fn bench_empty_space_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_empty_space_simple");
    let samples = [EMPTY; SampleShape::SIZE as usize];

    let mut buffer = UnitQuadBuffer::new();
    visible_block_faces(
        &samples,
        &SampleShape {},
        [0; 3],
        [17; 3],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );

    group.bench_with_input(
        BenchmarkId::from_parameter(format!("quads={}", buffer.num_quads())),
        &(),
        |b, _| {
            b.iter(|| {
                visible_block_faces(
                    &samples,
                    &SampleShape {},
                    [0; 3],
                    [17; 3],
                    &RIGHT_HANDED_Y_UP_CONFIG.faces,
                    &mut buffer,
                )
            });
        },
    );
    group.finish();
}

fn bench_sphere_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_sphere_simple");
    let mut samples = [EMPTY; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(16, SampleShape::delinearize(i));
        samples[i as usize] = sphere_voxel(p);
    }

    let mut buffer = UnitQuadBuffer::new();
    visible_block_faces(
        &samples,
        &SampleShape {},
        [0; 3],
        [17; 3],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );

    group.bench_with_input(
        BenchmarkId::from_parameter(format!("quads={}", buffer.num_quads())),
        &(),
        |b, _| {
            b.iter(|| {
                visible_block_faces(
                    &samples,
                    &SampleShape {},
                    [0; 3],
                    [17; 3],
                    &RIGHT_HANDED_Y_UP_CONFIG.faces,
                    &mut buffer,
                )
            });
        },
    );
    group.finish();
}

criterion_group!(
    benches,
    bench_sphere_simple,
    bench_sphere_greedy,
    bench_empty_space_simple,
    bench_empty_space_greedy
);
criterion_main!(benches);

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

fn sphere_voxel([x, y, z]: [f32; 3]) -> BoolVoxel {
    let d = x * x + y * y + z * z;

    if d > 0.9 {
        EMPTY
    } else {
        FULL
    }
}

fn into_domain(array_dim: u32, [x, y, z]: [u32; 3]) -> [f32; 3] {
    [
        (2.0 * x as f32 / array_dim as f32) - 1.0,
        (2.0 * y as f32 / array_dim as f32) - 1.0,
        (2.0 * z as f32 / array_dim as f32) - 1.0,
    ]
}
