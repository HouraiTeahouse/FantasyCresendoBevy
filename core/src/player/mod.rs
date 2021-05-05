pub type PlayerId = u8;

pub struct PlayerConfig {
    /// The player's selected character in a match.
    pub character_id: u32,
    /// The player's selected pallete.
    pub pallete: u8,
    /// The default damage the player starts with upon respawning.
    pub default_damage: f32,
}

pub struct Player {
    pub id: PlayerId,
}

pub enum PlayerDamage {
    Stock { stocks: u16, damage: f32 },
    Stamina { health: f32 },
}

impl PlayerDamage {
    /// Applies damage to the player.
    pub fn apply_damage(&mut self, dmg: f32) {
        match self {
            Self::Stock { damage, .. } => *damage += dmg,
            Self::Stamina { health } => {
                *health -= dmg;
                if *health < 0.0 {
                    *health = 0.0
                }
            }
        }
    }

    /// Forces the loss of one of the lives that the player has.
    /// For stock players, this will cause a loss of one stock.
    /// For stamina players, this will set their health to 0.
    pub fn kill(&mut self) {
        match self {
            Self::Stock { stocks, .. } => {
                if *stocks > 0_u16 {
                    *stocks -= 1;
                }
            }
            Self::Stamina { health } => *health = 0.0,
        }
    }

    /// Checks if the player can be revived normally.
    pub fn can_revive(&self) -> bool {
        match self {
            Self::Stock { stocks, .. } => *stocks > 0,
            Self::Stamina { health } => *health > 0.0,
        }
    }

    /// Resets a player after
    pub fn revive(&mut self, config: &PlayerConfig) {
        match self {
            Self::Stock { damage, .. } => *damage = config.default_damage,
            _ => {}
        }
    }
}
