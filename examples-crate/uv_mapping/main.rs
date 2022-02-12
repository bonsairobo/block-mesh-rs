use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::{AddressMode, PrimitiveTopology, SamplerDescriptor};
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum AppState {
    Loading,
    Run,
}

const UV_SCALE: f32 = 1.0 / 16.0;

struct Loading(Handle<Image>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(State::new(AppState::Loading))
        .add_state(AppState::Loading)
        .add_system_set(SystemSet::on_enter(AppState::Loading).with_system(load_assets))
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(check_loaded))
        .add_system_set(SystemSet::on_enter(AppState::Run).with_system(setup))
        .add_system_set(SystemSet::on_update(AppState::Run).with_system(camera_rotation_system))
        .run();
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    debug!("load");
    let handle = asset_server.load("uv_checker.png");
    commands.insert_resource(Loading(handle));
}

/// Make sure that our texture is loaded so we can change some settings on it later
fn check_loaded(
    mut state: ResMut<State<AppState>>,
    handle: Res<Loading>,
    asset_server: Res<AssetServer>,
) {
    debug!("check loaded");
    if let LoadState::Loaded = asset_server.get_load_state(&handle.0) {
        state.set(AppState::Run).unwrap();
    }
}

/// Basic voxel type with one byte of texture layers
#[derive(Default, Clone, Copy)]
struct BoolVoxel(bool);

impl MergeVoxel for BoolVoxel {
    type MergeValue = bool;

    fn merge_value(&self) -> Self::MergeValue {
        self.0
    }
}

impl Voxel for BoolVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.0 {
            VoxelVisibility::Opaque
        } else {
            VoxelVisibility::Empty
        }
    }
}

fn setup(
    mut commands: Commands,
    texture_handle: Res<Loading>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
) {
    debug!("setup");
    let mut texture = textures.get_mut(&texture_handle.0).unwrap();

    // Set the texture to tile over the entire quad
    texture.sampler_descriptor = SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        ..Default::default()
    };

    type SampleShape = ConstShape3u32<22, 22, 22>;

    // Just a solid cube of voxels. We only fill the interior since we need some empty voxels to form a boundary for the mesh.
    let mut voxels = [BoolVoxel(false); SampleShape::SIZE as usize];
    for z in 1..21 {
        for y in 1..21 {
            for x in 1..21 {
                let i = SampleShape::linearize([x, y, z]);
                voxels[i as usize] = BoolVoxel(true);
            }
        }
    }

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = GreedyQuadsBuffer::new(voxels.len());
    greedy_quads(
        &voxels,
        &SampleShape {},
        [0; 3],
        [21; 3],
        &faces,
        &mut buffer,
    );
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
            tex_coords.extend_from_slice(&face.tex_coords(
                RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
                true,
                &quad,
            ));
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    for uv in tex_coords.iter_mut() {
        for c in uv.iter_mut() {
            *c *= UV_SCALE;
        }
    }

    render_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    render_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    render_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    render_mesh.set_indices(Some(Indices::U32(indices)));

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(render_mesh),
        material: materials.add(texture_handle.0.clone().into()),
        transform: Transform::from_translation(Vec3::splat(-10.0)),
        ..Default::default()
    });

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 50.0)),
        point_light: PointLight {
            range: 200.0,
            intensity: 20000.0,
            ..Default::default()
        },
        ..Default::default()
    });
    let camera = commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .id();

    commands.insert_resource(CameraRotationState::new(camera));
}

struct CameraRotationState {
    camera: Entity,
}

impl CameraRotationState {
    fn new(camera: Entity) -> Self {
        Self { camera }
    }
}

fn camera_rotation_system(
    state: Res<CameraRotationState>,
    time: Res<Time>,
    mut transforms: Query<&mut Transform>,
) {
    let t = 0.3 * time.seconds_since_startup() as f32;

    let target = Vec3::new(0.0, 0.0, 0.0);
    let height = 30.0 * (2.0 * t).sin();
    let radius = 50.0;
    let x = radius * t.cos();
    let z = radius * t.sin();
    let eye = Vec3::new(x, height, z);
    let new_transform = Mat4::face_toward(eye, target, Vec3::Y);

    let mut cam_tfm = transforms.get_mut(state.camera).unwrap();
    *cam_tfm = Transform::from_matrix(new_transform);
}
