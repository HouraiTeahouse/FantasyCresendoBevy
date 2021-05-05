use bevy::prelude::*;
use fc_core::input::*;

struct Player;

fn create_players(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert(PlayerInput::default());
}

trait InputSource {
    fn update_input(&self, frame: &mut PlayerInputFrame);
}

impl<'w> InputSource for Res<'w, Input<KeyCode>> {
    fn update_input(&self, frame: &mut PlayerInputFrame) {
        fn keyboard_axis(keyboard: &Res<Input<KeyCode>>, pos: KeyCode, neg: KeyCode) -> Axis1D {
            Axis1D(match (keyboard.pressed(pos), keyboard.pressed(neg)) {
                (true, true) => 0_i8,
                (true, false) => i8::MAX,
                (false, true) => i8::MIN,
                (false, false) => 0_i8,
            })
        }

        let buttons = &mut frame.buttons;
        buttons.set_attack(self.pressed(KeyCode::F));
        buttons.set_special(self.pressed(KeyCode::D));
        buttons.set_shield(self.pressed(KeyCode::S) || self.pressed(KeyCode::A));
        buttons.set_jump(
            self.pressed(KeyCode::Q) || self.pressed(KeyCode::W) || self.pressed(KeyCode::I),
        );

        frame.movement = Axis2D {
            x: keyboard_axis(self, KeyCode::H, KeyCode::L),
            y: keyboard_axis(self, KeyCode::I, KeyCode::K),
        };
    }
}

fn sample_input(input: Res<Input<KeyCode>>, mut query: Query<&mut PlayerInput, With<Player>>) {
    for mut player_input in query.iter_mut() {
        player_input.tick();
        input.update_input(&mut player_input.current);
        if player_input.previous != player_input.current {
            println!("{:?}", player_input);
        }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(create_players.system())
        .add_system(sample_input.system())
        .run();
}
