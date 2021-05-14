use crate::r#match::player::PlayerBody;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::*,
    prelude::*,
};
pub use fc_core::{debug::DebugLines, geo::*};
use fc_core::{
    debug::DebugLinesPlugin,
    player::Player,
    stage::{BlastZone, RespawnPoint, SpawnPoint, Surface},
};

const CROSS_SIZE: f32 = 0.25;

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

fn update_fps_counter(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}

fn draw_player_debug(
    query: Query<(&Transform, &PlayerBody), With<Player>>,
    mut lines: ResMut<DebugLines>,
) {
    const SIZE: f32 = 0.25;
    let mut total_bounds: Option<Bounds2D> = None;
    for (transform, body) in query.iter() {
        let ecb = &body.ecb;
        let mut center = transform.translation;
        center.z = 0.0;
        lines.cross_2d(center, SIZE, Color::GRAY);

        let mut bounds = Bounds2D::from(ecb.clone());
        bounds.center += center.xy();
        bounds.center.y += ecb.bottom;

        if let Some(ref mut total) = total_bounds {
            total.merge_with(bounds);
        } else {
            total_bounds = Some(bounds);
        }

        let ecb_center = center + Vec3::new(0.0, ecb.bottom, 0.0);
        lines.polygon(
            [
                center,
                ecb_center + Vec3::new(-ecb.left, 0.0, 0.0),
                ecb_center + Vec3::new(0.0, ecb.top, 0.0),
                ecb_center + Vec3::new(ecb.right, 0.0, 0.0),
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
    for point in spawn.iter() {
        lines.cross_2d(Vec3::from((point.position, 0.0)), CROSS_SIZE, Color::YELLOW);
    }
    for point in respawn.iter() {
        let color = if point.occupied_by.is_none() {
            Color::CYAN
        } else {
            Color::RED
        };
        lines.cross_2d(point.position.extend(0.0), CROSS_SIZE, color);
    }
    for blast_zone in blast_zones.iter() {
        lines.bounds_2d(blast_zone.0, Color::MAROON);
    }
    for surface in surfaces.iter() {
        let start = surface.start.point.extend(0.0);
        let end = surface.end.point.extend(0.0);
        lines.line_colored(start, end, Color::WHITE);
    }
}

pub struct FcDebugPlugin;

impl Plugin for FcDebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(DebugLinesPlugin)
            .add_startup_system(start_debug.system())
            .add_system(update_fps_counter.system())
            .add_system(draw_player_debug.system())
            .add_system(draw_stage_debug.system());
    }
}
