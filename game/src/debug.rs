use crate::r#match::player::{EnvironmentCollisionBox, Player};
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::*,
    prelude::*,
};
use fc_core::debug::DebugLinesPlugin;
pub use fc_core::{debug::DebugLines, geo::*};

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
    query: Query<(&Transform, &EnvironmentCollisionBox), With<Player>>,
    mut lines: ResMut<DebugLines>,
) {
    const SIZE: f32 = 0.25;
    let mut total_bounds: Option<Bounds2D> = None;
    for (transform, ecb) in query.iter() {
        let mut center = transform.translation;
        center.z = 0.0;
        lines.line_colored(
            center + Vec3::new(-SIZE, 0.0, 0.0),
            center + Vec3::new(SIZE, 0.0, 0.0),
            Color::GRAY,
        );
        lines.line_colored(
            center + Vec3::new(0.0, -SIZE, 0.0),
            center + Vec3::new(0.0, SIZE, 0.0),
            Color::GRAY,
        );

        let mut bounds = Bounds2D::from(ecb.clone());
        bounds.center += center.xy();
        bounds.center.y += ecb.bottom;
        lines.bounds_2d(bounds, Color::GREEN);

        if let Some(ref mut total) = total_bounds {
            total.merge_with(bounds);
        } else {
            total_bounds = Some(bounds);
        }

        let ecb_bottom = center;
        let ecb_center = center + Vec3::new(0.0, ecb.bottom, 0.0);
        let ecb_top = ecb_center + Vec3::new(0.0, ecb.top, 0.0);
        let ecb_left = ecb_center + Vec3::new(-ecb.left, 0.0, 0.0);
        let ecb_right = ecb_center + Vec3::new(ecb.right, 0.0, 0.0);

        lines.line_colored(ecb_bottom, ecb_right, Color::YELLOW);
        lines.line_colored(ecb_bottom, ecb_left, Color::YELLOW);
        lines.line_colored(ecb_top, ecb_right, Color::YELLOW);
        lines.line_colored(ecb_top, ecb_left, Color::YELLOW);
    }

    if let Some(bounds) = total_bounds {
        lines.bounds_2d(bounds, Color::CYAN);
    }
}

pub struct FcDebugPlugin;

impl Plugin for FcDebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(DebugLinesPlugin)
            .add_startup_system(start_debug.system())
            .add_system(update_fps_counter.system())
            .add_system(draw_player_debug.system());
    }
}
