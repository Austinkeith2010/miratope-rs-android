use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::camera::Camera;
use bevy::render::pipeline::PipelineDescriptor;
use no_cull_pipeline::PbrNoBackfaceBundle;
use polytope::*;
use polytope::{off::polytope_from_off_src, shapes::*};

mod no_cull_pipeline;
mod polytope;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(spin_camera.system())
        .run();
}

const WIREFRAME_SELECTED_MATERIAL: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 0x82A3A5DD3A34CC21);
const WIREFRAME_UNSELECTED_MATERIAL: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 0x82A3A5DD3A34CC22);

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
) {
    let cube_off = "OFF
    8 6 1337
    
    # Vertices
    0.5 0.5 0.5 #this
    0.5 0.5 -0.5 #   is
    0.5 -0.5 0.5 # a
    0.5 -0.5 -0.5 #test
    -0.5 0.5 0.5 # of
    -0.5 0.5 -0.5 #the new
    -0.5 -0.5 0.5     #comment 
    -0.5 -0.5 -0.5 #removal
    # system
    # Faces
    4 4 0 2 6
    4 0 1 3 2
    4 6 7 3 2
    4 5 7 6 4
    4 4 0 1 5
    4 7 5 1 3";
    let poly: Polytope = polytope_from_off_src(cube_off.to_string()).into();

    pipelines.set_untracked(
        no_cull_pipeline::NO_CULL_PIPELINE_HANDLE,
        no_cull_pipeline::build_no_cull_pipeline(&mut shaders),
    );

    materials.set_untracked(
        WIREFRAME_SELECTED_MATERIAL,
        Color::rgb_u8(126, 192, 236).into(),
    );

    let wf_unselected = materials.set(
        WIREFRAME_UNSELECTED_MATERIAL,
        Color::rgb_u8(56, 68, 236).into(),
    );

    commands
        .spawn(PbrNoBackfaceBundle {
            mesh: meshes.add(poly.get_mesh()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
            ..Default::default()
        })
        .with_children(|cb| {
            cb.spawn(PbrNoBackfaceBundle {
                mesh: meshes.add(poly.get_wireframe()),
                material: wf_unselected,
                ..Default::default()
            });
        })
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 2.0)),
            ..Default::default()
        })
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 5.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}

fn spin_camera(mut query: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    const SPIN_RATE: f32 = std::f32::consts::PI * 2.0 / 3.0;

    for mut tf in query.iter_mut() {
        tf.translation = Quat::from_rotation_y(time.delta_seconds() * SPIN_RATE) * tf.translation;
        tf.look_at(Vec3::zero(), Vec3::unit_y());
    }
}
