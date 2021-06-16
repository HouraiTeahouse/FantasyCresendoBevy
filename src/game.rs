#![allow(clippy::float_cmp)]

#[macro_use]
extern crate bitflags;

use crate::{player, r#match::input::*};
#[windows_subsystem = "windows"]
use bevy::prelude::*;
use bevy_backroll::backroll;
use backroll_transport_udp::*;
use bevy_steamworks::{AppId, SteamworksPlugin};
use bevy::tasks::IoTaskPool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;

mod assets;
mod character;
#[cfg(debug_assertions)]
mod debug;
mod geo;
mod input;
mod r#match;
mod time;

use r#match::*;

const STEAM_APP_ID: AppId = AppId(774701);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppState {
    STARTUP,
    MATCH,
}

fn create_input_source(arrow: ButtonAxis2D<KeyCode>, jump: KeyCode) -> InputSource {
    let mut buttons: HashMap<Buttons, Vec<KeyCode>> = HashMap::new();
    buttons.insert(Buttons::ATTACK, vec![KeyCode::F]);
    buttons.insert(Buttons::SPECIAL, vec![KeyCode::D]);
    buttons.insert(Buttons::JUMP, vec![jump]);
    buttons.insert(Buttons::SHIELD, vec![KeyCode::Q, KeyCode::W]);
    InputSource::Keyboard {
        movement: arrow.clone(),
        smash: arrow.clone(),
        buttons: ButtonMapping::<KeyCode>(buttons),
    }
}

fn main() {
    // Restart the game if need be through Steam, otherwise set the AppId
    // to ensure proper initialzation.
    #[cfg(feature = "steam-restart")]
    if bevy_steamworks::restart_app_if_necessary(STEAM_APP_ID) {
        return;
    }

    #[cfg(not(feature = "steam-restart"))]
    {
        let app_id = STEAM_APP_ID.0.to_string();
        std::env::set_var("SteamAppId", &app_id);
        std::env::set_var("SteamGameId", app_id);
    }

    let mut args = std::env::args();
    let base = args.next().unwrap();
    if let Some(player_num) = args.next() {
        start_app(player_num.parse().unwrap());
    } else {
        let mut child_1 = std::process::Command::new(base.clone())
            .args(&["0"])
            .spawn()
            .unwrap();
        let mut child_2 = std::process::Command::new(base)
            .args(&["1"])
            .spawn()
            .unwrap();
        child_1.wait();
        child_2.wait();
    }
}

#[derive(Debug)]
struct StartupConfig {
    client: usize,
    bind: SocketAddr,
    remote: SocketAddr,
}

fn start_app(player_num: usize) {
    let bind_addr = if player_num == 0 {
        "127.0.0.1:4001".parse().unwrap()
    } else {
        "127.0.0.1:4002".parse().unwrap()
    };

    let remote_addr = if player_num == 0 {
        "127.0.0.1:4002".parse().unwrap()
    } else {
        "127.0.0.1:4001".parse().unwrap()
    };

    let mut app = App::build();
    app.insert_resource(StartupConfig {
        client: player_num,
        bind: bind_addr,
        remote: remote_addr,
    })
    .insert_resource(WindowDescriptor {
        title: "Fantasy Crescendo".to_string(),
        vsync: true,
        ..Default::default()
    })
    .add_state(AppState::STARTUP)
    .add_plugins_with(DefaultPlugins, |cfg| {
        if player_num != 0 {
            cfg.disable::<bevy::log::LogPlugin>();
        }
        cfg
    })
    .add_plugin(SteamworksPlugin)
    .add_plugin(input::FcInputPlugin)
    .add_plugin(assets::FcAssetsPlugin)
    .add_plugin(r#match::FcMatchPlugin)
    .insert_resource(Msaa { samples: 1 })
    .add_startup_system(setup.system())
    .add_system(events.system());

    // Optional Plugins
    #[cfg(debug_assertions)]
    app.add_plugin(debug::FcDebugPlugin);

    app.run();
}

fn events(mut events: EventReader<backroll::Event>) {
    for event in events.iter() {
        info!("{:?}", event);
    }
}

/// set up a simple 3D scene
fn setup(
    config: Res<StartupConfig>, 
    pool: Res<IoTaskPool>,
    mut commands: Commands) {
    // cameras
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        perspective_projection: bevy::render::camera::PerspectiveProjection {
            fov: 60.0 * std::f32::consts::PI / 180.0,
            ..Default::default()
        },
        ..Default::default()
    });

    println!("{:?}", config);
    let socket = UdpManager::bind(pool.deref().deref().clone(), config.bind).unwrap();
    let peer = socket.connect(UdpConnectionConfig::unbounded(config.remote));
    let remote = backroll::Player::Remote(peer);

    commands.insert_resource(socket);
    commands.insert_resource(MatchConfig {
        rule: r#match::rule::MatchRule::Stock(3),
        time: None,
        players: [
            Some(player::PlayerConfig {
                player: if config.client == 0 {
                    backroll::Player::Local
                } else {
                    remote.clone()
                },
                character_id: 0,
                pallete: 0,
                default_damage: 0.0,
                input: create_input_source(
                    ButtonAxis2D::<KeyCode> {
                        horizontal: ButtonAxis1D::<KeyCode> {
                            pos: KeyCode::D,
                            neg: KeyCode::A,
                        },
                        vertical: ButtonAxis1D::<KeyCode> {
                            pos: KeyCode::W,
                            neg: KeyCode::S,
                        },
                    },
                    KeyCode::W,
                ),
            }),
            Some(player::PlayerConfig {
                player: if config.client == 1 {
                    backroll::Player::Local
                } else {
                    remote
                },
                character_id: 0,
                pallete: 0,
                default_damage: 0.0,
                input: create_input_source(
                    ButtonAxis2D::<KeyCode> {
                        horizontal: ButtonAxis1D::<KeyCode> {
                            pos: KeyCode::L,
                            neg: KeyCode::J,
                        },
                        vertical: ButtonAxis1D::<KeyCode> {
                            pos: KeyCode::I,
                            neg: KeyCode::K,
                        },
                    },
                    KeyCode::I,
                ),
            }),
            None,
            None,
        ],
    });
}
