mod capsule;
mod line;

use self::capsule::{Capsule, CapsuleGenerator, DebugCapsulesPlugin};
use self::line::{DebugLines, DebugLinesPlugin};

use crate::{
    character::frame_data::{hitbox::Hitbox, hurtbox::Hurtbox},
    geo::*,
    player::Player,
    r#match::{
        hitbox::HitboxState,
        physics::Body,
        stage::{BlastZone, RespawnPoint, SpawnPoint, Surface},
    },
};
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::*,
    prelude::*,
};

const CROSS_SIZE: f32 = 0.25;
const HITBOX_ALPHA: f32 = 0.25;

fn start_debug(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        },
        text: Text {
            sections: vec![
                TextSection {
                    value: "FPS: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 15.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 15.0,
                        color: Color::GOLD,
                    },
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    });
}

fn update_fps_counter(diagnostics: Res<Diagnostics>, mut texts: Query<&mut Text>) {
    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average());
    if let Some(fps) = fps {
        texts.for_each_mut(|mut text| {
            text.sections[1].value = format!("{:.2}", fps);
        });
    }
}

fn draw_player_debug(
    bodies: Query<(&Transform, &Body), With<Player>>,
    mut lines: ResMut<DebugLines>,
) {
    const SIZE: f32 = 0.25;
    let mut total_bounds: Option<Bounds2D> = None;
    for (transform, body) in bodies.iter() {
        let mut ecb = body.ecb.clone();
        let mut center = transform.translation;
        center.z = 0.0;
        lines.directed_cross_2d(center, SIZE, Color::GRAY, body.facing);

        ecb.translate(center.xy() - ecb.bottom());

        if let Some(ref mut total) = total_bounds {
            total.merge_with(ecb.0);
        } else {
            total_bounds = Some(ecb.0);
        }

        lines.polygon(
            [
                ecb.bottom().extend(0.0),
                ecb.left().extend(0.0),
                ecb.top().extend(0.0),
                ecb.right().extend(0.0),
            ]
            .iter()
            .cloned(),
            Color::YELLOW,
        );
    }

    if let Some(bounds) = total_bounds {
        lines.bounds_2d(bounds, Color::CYAN);
    }
}

fn draw_stage_debug(
    spawn: Query<&SpawnPoint>,
    respawn: Query<&RespawnPoint>,
    blast_zones: Query<&BlastZone>,
    surfaces: Query<&Surface>,
    mut lines: ResMut<DebugLines>,
) {
    spawn.for_each(|point| {
        lines.cross_2d(Vec3::from((point.position, 0.0)), CROSS_SIZE, Color::YELLOW);
    });
    respawn.for_each(|point| {
        let color = match point.occupied_by {
            Some(_) => Color::RED,
            None => Color::CYAN,
        };
        lines.cross_2d(point.position.extend(0.0), CROSS_SIZE, color);
    });
    blast_zones.for_each(|zone| {
        lines.bounds_2d(zone.0, Color::MAROON);
    });
    surfaces.for_each(|surface| {
        let start = surface.start.point.extend(0.0);
        let end = surface.end.point.extend(0.0);
        lines.line_colored(start, end, Color::WHITE);
    });
}

fn visualize_hitboxes(
    mut commands: Commands,
    generator: Res<CapsuleGenerator>,
    hitboxes: Query<Entity, (With<Hitbox>, Without<Capsule>)>,
) {
    hitboxes.for_each(|hitbox| {
        commands.entity(hitbox).insert_bundle(generator.create());
    });
}

fn visualize_hurtboxes(
    mut commands: Commands,
    generator: Res<CapsuleGenerator>,
    hurtboxes: Query<Entity, (With<Hurtbox>, Without<Capsule>)>,
) {
    hurtboxes.for_each(|hitbox| {
        commands.entity(hitbox).insert_bundle(generator.create());
    });
}

fn update_hitbox_debug(
    mut hitboxes: Query<(
        &Hitbox,
        &HitboxState,
        &GlobalTransform,
        &mut Capsule,
        &mut Visible,
    )>,
) {
    hitboxes.for_each_mut(|(hitbox, state, transform, mut capsule, mut visible)| {
        visible.is_visible = state.enabled;
        capsule.start = transform.translation;
        capsule.end = state.previous_position.unwrap_or(transform.translation);
        capsule.radius = hitbox.radius;
        capsule.color = hitbox.color();
        capsule.color.set_a(HITBOX_ALPHA);
    });
}

fn update_hurtbox_debug(
    mut hitboxes: Query<(&Hurtbox, &GlobalTransform, &mut Capsule, &mut Visible)>,
) {
    hitboxes.for_each_mut(|(hurtbox, transform, mut capsule, mut visible)| {
        let local_to_world = transform.compute_matrix();
        visible.is_visible = hurtbox.is_enabled();
        capsule.start = local_to_world.transform_point3(hurtbox.collider.start);
        capsule.end = local_to_world.transform_point3(hurtbox.collider.end);
        capsule.radius = transform.scale.max_element() * hurtbox.collider.radius;
        capsule.color = hurtbox.r#type.color();
        capsule.color.set_a(HITBOX_ALPHA);
    });
}

pub struct FcDebugPlugin;

impl Plugin for FcDebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(DebugLinesPlugin)
            .add_plugin(DebugCapsulesPlugin)
            .add_startup_system(start_debug.system())
            .add_system(update_fps_counter.system())
            .add_system(draw_player_debug.system())
            .add_system(draw_stage_debug.system())
            .add_system(visualize_hitboxes.system())
            .add_system(visualize_hurtboxes.system())
            .add_system(update_hitbox_debug.system())
            .add_system(update_hurtbox_debug.system());
    }
}
