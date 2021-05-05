use crate::PlayerId;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HurtboxType {
    Inactive,
    Damageable,
    Intangible,
    Invincible,
    Grazing,
    Shield,
}

#[derive(Clone, Copy, Debug)]
pub struct Hurtbox {
    pub id: u8,
    pub player: PlayerId,
    pub r#type: HurtboxType,
}

impl Hurtbox {
    pub fn is_enabled(&self) -> bool {
        self.r#type != HurtboxType::Inactive
    }
}
