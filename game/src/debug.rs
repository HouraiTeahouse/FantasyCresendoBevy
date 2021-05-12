use crate::r#match::player::{EnvironmentCollisionBox, Player};
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
pub use fc_core::debug::DebugLines;
use fc_core::debug::DebugLinesPlugin;

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
    query: Query<(&GlobalTransform, &EnvironmentCollisionBox), With<Player>>,
    mut lines: ResMut<DebugLines>,
) {
    const SIZE: f32 = 0.25;
    for (transform, ecb) in query.iter() {
        let center = transform.translation;
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
