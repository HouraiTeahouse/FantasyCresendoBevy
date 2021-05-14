use super::{hitbox, MatchRule};
use bevy::prelude::*;
use fc_core::{
    character::{frame_data::*, state::*},
    geo::*,
    input::*,
    player::*,
};
use serde::{Deserialize, Serialize};

const PLAYER_COLORS: &[Color] = &[Color::RED, Color::BLUE, Color::YELLOW, Color::GREEN];

#[derive(Clone, Deserialize, Serialize)]
pub struct PlayerConfig {
    /// The player's selected character in a match.
    pub character_id: u32,
    /// The player's selected pallete.
    pub pallete: u8,
    /// The default damage the player starts with upon respawning.
    pub default_damage: f32,
    /// The default damage the player starts with upon respawning.
    #[serde(skip)]
    pub input: InputSource,
}

#[derive(Debug, Clone)]
pub(super) enum PlayerDamage {
    Score {
        score: i16,
        damage: f32,
        default_damage: f32,
    },
    Stock {
        stocks: u8,
        damage: f32,
        default_damage: f32,
    },
    Stamina {
        health: f32,
        full_health: f32,
    },
}

impl Default for PlayerDamage {
    fn default() -> Self {
        Self::Stock {
            stocks: 0,
            damage: 0.0,
            default_damage: 0.0,
        }
    }
}

impl PlayerDamage {
    pub const MIN: f32 = 0.0;
    pub const MAX: f32 = 999.99;

    pub fn new(rule: &MatchRule, config: &PlayerConfig) -> Self {
        match rule {
            MatchRule::Score => Self::Score {
                score: 0,
                damage: config.default_damage,
                default_damage: config.default_damage,
            },
            MatchRule::Stock(stocks) => Self::Stock {
                stocks: *stocks,
                damage: config.default_damage,
                default_damage: config.default_damage,
            },
            MatchRule::Stamina(health) => Self::Stamina {
                health: *health,
                full_health: *health,
            },
        }
    }

    pub fn knockback_scaling(&self) -> f32 {
        match self {
            Self::Score { damage, .. } => *damage,
            Self::Stock { damage, .. } => *damage,
            Self::Stamina { .. } => 0.0,
        }
    }

    /// Applies damage to the player.
    pub fn apply_damage(&mut self, dmg: f32) {
        match self {
            Self::Score { damage, .. } => *damage = Self::clamp(*damage + dmg),
            Self::Stock { damage, .. } => *damage = Self::clamp(*damage + dmg),
            Self::Stamina { health, .. } => {
                *health -= dmg;
                if *health < 0.0 {
                    *health = 0.0
                }
            }
        }
    }

    /// Checks if the player is alive.
    pub fn is_alive(&self) -> bool {
        match self {
            Self::Score { .. } => true,
            Self::Stock { stocks, .. } => *stocks > 0,
            Self::Stamina { health, .. } => *health > 0.0,
        }
    }

    /// Forces the loss of one of the lives that the player has.
    /// For stock players, this will cause a loss of one stock.
    /// For stamina players, this will set their health to 0.
    pub fn kill(&mut self) {
        match self {
            Self::Score { score, .. } => {
                *score += 1;
            }
            Self::Stock { stocks, .. } => {
                if *stocks > 0 {
                    *stocks -= 1;
                }
            }
            Self::Stamina { health, .. } => *health = 0.0,
        }
    }

    /// Checks if the player can be revived normally.
    pub fn can_revive(&self) -> bool {
        match self {
            Self::Score { .. } => true,
            Self::Stock { stocks, .. } => *stocks > 0,
            Self::Stamina { .. } => false,
        }
    }

    /// Resets a player after they've been killed.
    pub fn revive(&mut self) {
        match self {
            Self::Score {
                damage,
                default_damage,
                ..
            } => *damage = Self::clamp(*default_damage),
            Self::Stock {
                damage,
                default_damage,
                ..
            } => *damage = Self::clamp(*default_damage),
            Self::Stamina {
                health,
                full_health,
                ..
            } => *health = Self::clamp(*full_health),
        }
    }

    fn clamp(dmg: f32) -> f32 {
        f32::clamp(dmg, Self::MIN, Self::MAX)
    }
}

#[derive(Debug, Default)]
pub struct PlayerBody {
    pub mass: f32,
    pub facing: Facing,
    pub location: PlayerLocation,
    pub velocity: Vec2,
    pub ecb: EnvironmentCollisionBox,
}

#[derive(Debug)]
pub enum PlayerLocation {
    Airborne(Vec2),
    Respawning(Entity),
}

impl Default for PlayerLocation {
    fn default() -> Self {
        Self::Airborne(Vec2::ZERO)
    }
}

#[derive(Bundle, Default)]
pub(super) struct PlayerBundle {
    pub player: Player,
    pub body: PlayerBody,
    pub input: PlayerInput,
    pub damage: PlayerDamage,
    pub input_source: InputSource,
    #[bundle]
    pub character: CharacterBundle,
    #[bundle]
    pub pbr: PbrBundle,
}

#[derive(Bundle, Default)]
pub(super) struct CharacterBundle {
    pub state: PlayerState,
    pub state_machine: StateMachine,
    pub frame: CharacterFrame,
}

#[derive(Debug, Default, Clone)]
pub struct EnvironmentCollisionBox {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl From<EnvironmentCollisionBox> for Bounds2D {
    fn from(value: EnvironmentCollisionBox) -> Self {
        Self::from(Rect::<f32> {
            left: -value.left,
            right: value.right,
            top: -value.top,
            bottom: value.bottom,
        })
    }
}

pub(super) fn spawn_player(commands: &mut Commands, bundle: PlayerBundle) -> Entity {
    let player_id = bundle.player.id;
    commands
        .spawn_bundle(bundle)
        .with_children(|parent| {
            for bundle in hitbox::create_player_hitboxes(player_id) {
                parent.spawn_bundle(bundle);
            }
        })
        .id()
}

pub fn get_player_color(player: PlayerId) -> Color {
    PLAYER_COLORS[player as usize % PLAYER_COLORS.len()]
}
