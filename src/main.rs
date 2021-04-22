#![allow(dead_code)]
//! A renderer for polytopes, spinned off from [Miratope JS](https://github.com/OfficialURL/miratope).
//! Still in alpha development.
//!
//! ## What can Miratope do now?
//! Miratope can already load some polytopes and find out various properties
//! about them, and it can operate on them via various methods. We're still in
//! the early stages of porting the original Miratope's functionality, though.
//!
//! ## What are Miratope's goals?
//! We plan to eventually support all of the original Miratope's features,
//! as well as the following:
//!
//! * Various families of polytopes to build and render
//!   * All [regular polytopes](https://polytope.miraheze.org/wiki/Regular_polytope)
//!   * All known 3D and 4D [uniform polytopes](https://polytope.miraheze.org/wiki/Uniform_polytope)
//!   * Many of the known [CRFs](https://polytope.miraheze.org/wiki/Convex_regular-faced_polytope)
//! * Many operations to apply to these polytopes
//!   * [Duals](https://polytope.miraheze.org/wiki/Dual)
//!   * [Petrials](https://polytope.miraheze.org/wiki/Petrial)
//!   * [Prism products](https://polytope.miraheze.org/wiki/Prism_product)
//!   * [Tegum products](https://polytope.miraheze.org/wiki/Tegum_product)
//!   * [Pyramid products](https://polytope.miraheze.org/wiki/Pyramid_product)
//!   * [Convex hulls](https://polytope.miraheze.org/wiki/Convex_hull)
//! * Loading and saving into various formats
//!   * Support for the [Stella OFF format](https://www.software3d.com/StellaManual.php?prod=stella4D#import)
//!   * Support for the [GeoGebra GGB format](https://wiki.geogebra.org/en/Reference:File_Format)
//! * Localization
//!   * Automatic name generation in various languages for many shapes
//!
//! ## How do I use Miratope?
//! Miratope doesn't have a very good interface yet, so you'll have to download
//! the source code to do much of anything.
//!
//! ## Where do I get these "OFF files"?
//! The OFF file format is a format for storing certain kinds of geometric
//! shapes. Although not in widespread use, it has become the standard format
//! for those who investigate polyhedra and polytopes. It was initially meant
//! for the [Geomview software](https://people.sc.fsu.edu/~jburkardt/data/off/off.html),
//! and was later adapted for the [Stella software](https://www.software3d.com/StellaManual.php?prod=stella4D#import).
//! Miratope uses a further generalization of the Stella OFF format for any
//! amount of dimensions.
//!
//! Miratope does not yet include a library of OFF files. Nevertheless, many of
//! them can be downloaded from [OfficialURL's personal collection](https://drive.google.com/drive/u/0/folders/1nQZ-QVVBfgYSck4pkZ7he0djF82T9MVy).
//! Eventually, they'll be browsable from Miratope itself.
//!
//! ## Why is the rendering buggy?
//! Proper rendering, even in 3D, is a work in progress.

use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::{camera::PerspectiveProjection, pipeline::PipelineDescriptor};
use bevy_egui::EguiPlugin;
use no_cull_pipeline::PbrNoBackfaceBundle;

#[allow(unused_imports)]
use polytope::{geometry::*, group::*, off::*, Polytope, *};
#[allow(unused_imports)]
use translation::{lang, Language, Options};
use ui::{input::CameraInputEvent, CrossSectionActive, CrossSectionState};

mod no_cull_pipeline;
mod polytope;
mod translation;
mod ui;

/// Standard constant used for floating point comparisons throughout the code.
const EPS: f64 = 1e-9;

/// Loads all of the necessary systems for the application to run.
fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_resource(Msaa { samples: 4 })
        .add_resource(CrossSectionActive(false))
        .add_resource(CrossSectionState::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(ui::input::InputPlugin)
        .add_startup_system(setup.system())
        .add_system(ui::update_scale_factor.system())
        .add_system_to_stage(stage::PRE_UPDATE, ui::ui.system())
        .add_system_to_stage(stage::UPDATE, ui::update_cross_section_state.system())
        .add_system_to_stage(stage::POST_UPDATE, ui::update_cross_section.system())
        .add_system_to_stage(stage::POST_UPDATE, ui::update_changed_polytopes.system())
        .run();
}

/// Initializes the scene.
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
) {
    let p = Concrete::point().prism().prism().prism();
    dbg!(lang::En::parse(p.name(), Options::default()));
    dbg!(lang::En::parse(p.abs.name(), Options::default()));
    let poly = Renderable::new(p);

    pipelines.set_untracked(
        no_cull_pipeline::NO_CULL_PIPELINE_HANDLE,
        no_cull_pipeline::build_no_cull_pipeline(&mut shaders),
    );

    materials.set_untracked(
        WIREFRAME_SELECTED_MATERIAL,
        Color::rgb_u8(126, 192, 255).into(),
    );

    let wf_unselected = materials.set(
        WIREFRAME_UNSELECTED_MATERIAL,
        Color::rgb_u8(238, 130, 238).into(),
    );

    let mut cam_anchor = Transform::default();
    let mut cam = Transform::default();
    CameraInputEvent::reset(&mut cam_anchor, &mut cam);

    commands
        .spawn(PbrNoBackfaceBundle {
            mesh: meshes.add(poly.get_mesh()),
            visible: Visible {
                is_visible: false,
                ..Default::default()
            },
            material: materials.add(Color::rgb(0.93, 0.5, 0.93).into()),
            ..Default::default()
        })
        .with_children(|cb| {
            cb.spawn(PbrNoBackfaceBundle {
                mesh: meshes.add(poly.get_wireframe()),
                material: wf_unselected,
                ..Default::default()
            });
        })
        .with(poly)
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 2.0)),
            ..Default::default()
        })
        // camera anchor
        .spawn((GlobalTransform::default(), cam_anchor))
        .with_children(|cb| {
            // camera
            cb.spawn(Camera3dBundle {
                transform: cam,
                perspective_projection: PerspectiveProjection {
                    near: 0.0001,
                    ..Default::default()
                },
                ..Default::default()
            });
        });
}

const WIREFRAME_SELECTED_MATERIAL: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 0x82A3A5DD3A34CC21);
const WIREFRAME_UNSELECTED_MATERIAL: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 0x82A3A5DD3A34CC22);
