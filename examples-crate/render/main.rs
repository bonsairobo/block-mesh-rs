use block_mesh::ilattice::glam::Vec3A;
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{
    greedy_quads, visible_block_faces, GreedyQuadsBuffer, MergeVoxel, UnitQuadBuffer, Voxel, VoxelVisibility,
    RIGHT_HANDED_Y_UP_CONFIG,
};

use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        options::WgpuOptions,
        render_resource::{PrimitiveTopology, WgpuFeatures},
    },
};

fn main() {
    App::new()
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    wireframe_config.global = true;

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(25.0, 25.0, 25.0)),
        point_light: PointLight {
            range: 200.0,
            intensity: 8000.0,
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(50.0, 15.0, 50.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });

    let simple_sphere_mesh = generate_simple_mesh(&mut meshes, |p| sphere(0.9, p));
    let greedy_sphere_mesh = generate_greedy_mesh(&mut meshes, |p| sphere(0.9, p));

    spawn_pbr(
        &mut commands,
        &mut materials,
        simple_sphere_mesh,
        Transform::from_translation(Vec3::new(8.0, -16.0, -16.0)),
    );
    spawn_pbr(
        &mut commands,
        &mut materials,
        greedy_sphere_mesh,
        Transform::from_translation(Vec3::new(-16.0, -16.0, 8.0)),
    );
}

fn generate_simple_mesh(
    meshes: &mut Assets<Mesh>,
    sdf: impl Fn(Vec3A) -> BoolVoxel,
) -> Handle<Mesh> {
    type SampleShape = ConstShape3u32<34, 34, 34>;

    let mut samples = [EMPTY; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(32, SampleShape::delinearize(i));
        samples[i as usize] = sdf(p);
    }

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = UnitQuadBuffer::new();
    visible_block_faces(
        &samples,
        &SampleShape {},
        [0; 3],
        [33; 3],
        &faces,
        &mut buffer,
    );
    let num_indices = buffer.num_quads() * 6;
    let num_vertices = buffer.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.set_attribute(
        "Vertex_Position",
        VertexAttributeValues::Float32x3(positions),
    );
    render_mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float32x3(normals));
    render_mesh.set_attribute(
        "Vertex_Uv",
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );
    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    meshes.add(render_mesh)
}

fn generate_greedy_mesh(
    meshes: &mut Assets<Mesh>,
    sdf: impl Fn(Vec3A) -> BoolVoxel,
) -> Handle<Mesh> {
    type SampleShape = ConstShape3u32<34, 34, 34>;

    let mut samples = [EMPTY; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(32, SampleShape::delinearize(i));
        samples[i as usize] = sdf(p);
    }

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = GreedyQuadsBuffer::new(samples.len());
    greedy_quads(
        &samples,
        &SampleShape {},
        [0; 3],
        [33; 3],
        &faces,
        &mut buffer,
    );
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.set_attribute(
        "Vertex_Position",
        VertexAttributeValues::Float32x3(positions),
    );
    render_mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float32x3(normals));
    render_mesh.set_attribute(
        "Vertex_Uv",
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );
    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    meshes.add(render_mesh)
}

fn spawn_pbr(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: Handle<Mesh>,
    transform: Transform,
) {
    let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
    material.perceptual_roughness = 0.9;

    commands.spawn_bundle(PbrBundle {
        mesh,
        material: materials.add(material),
        transform,
        ..Default::default()
    });
}

fn into_domain(array_dim: u32, [x, y, z]: [u32; 3]) -> Vec3A {
    (2.0 / array_dim as f32) * Vec3A::new(x as f32, y as f32, z as f32) - 1.0
}

fn sphere(radius: f32, p: Vec3A) -> BoolVoxel {
    BoolVoxel(p.length() < radius)
}

#[derive(Clone, Copy, Eq, PartialEq)]
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

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}
