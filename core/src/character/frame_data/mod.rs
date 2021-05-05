use bevy_math::Vec2;
use serde::{Deserialize, Serialize};

pub mod hitbox;
pub mod hurtbox;

bitflags! {
    #[derive(Default, Deserialize, Serialize)]
    pub struct CharacterFrameFlags : u8 {
        /// If set, all of a character's hitboxes will be intangible for the frame.
        const INTANGIBLE = 1 << 0;
        /// If set, all of a character's hitboxes will be grazing for the frame.
        const GRAZING = 1 << 1;
        /// If set, the character will change to face left before the start of the frame.
        const FACE_LEFT = 1 << 2;
        /// If set, the character will change to face right before the start of the frame.
        const FACE_RIGHT = 1 << 3;
        /// If set, the character will change to face right before the start of the frame.
        const CHANGE_DIRECTION = 1 << 4;
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CharacterFrame {
    pub flags: CharacterFrameFlags,
    /// Bitfield where 1s demark an active hitbox.
    /// Supports up to unique 32 hitboxes per state.
    pub active_hitboxes: u32,
    /// Forced character movement per frame.
    pub movement: Vec2,
    /// A flat amount of damage subtracted from all damage taken by the player during this
    /// frame. If infinite, the player is invincible.
    pub damage_resistance: f32,
    /// A flat amount of knockback force subtracted from all knockback dealt to the player
    /// during this frame. If infinite, the player has super armor.
    pub knockback_resistance: f32,
}

impl CharacterFrame {
    /// Checks if a hitbox is active in a given frame.
    pub fn is_hitbox_active(&self, hitbox: u8) -> bool {
        (self.active_hitboxes & (1 << hitbox)) != 0
    }
}

/// The full hitbox frame data for a given state in a character.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateFrameData {
    pub hitboxes: Vec<hitbox::Hitbox>,
    pub frames: Vec<CharacterFrame>,
}

impl StateFrameData {
    /// Checks if a hitbox is active on a given frame. If
    pub fn get_frame(&self, frame: usize) -> Option<&CharacterFrame> {
        self.frames.get(frame)
    }

    /// Like [`get_frame`] but loops back around to the start of the state if the frame
    /// exceeds the frame length of the state is exceeded.
    pub fn get_frame_looped(&self, frame: usize) -> Option<&CharacterFrame> {
        self.frames.get(frame % self.frames.len())
    }
}
