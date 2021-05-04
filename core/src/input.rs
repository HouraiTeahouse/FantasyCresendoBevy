use std::fmt;

bitflags! {
    #[derive(Default)]
    pub struct Buttons: u8 {
        const ATTACK = 1 << 0;
        const SPECIAL = 1 << 1;
        const JUMP = 1 << 2;
        const SHIELD = 1 << 3;
        const GRAB = 1 << 4;
    }
}

impl Buttons {
    #[inline]
    pub fn attack(&self) -> bool {
        self.contains(Self::ATTACK)
    }

    #[inline]
    pub fn special(&self) -> bool {
        self.contains(Self::SPECIAL)
    }

    #[inline]
    pub fn jump(&self) -> bool {
        self.contains(Self::JUMP)
    }

    #[inline]
    pub fn shield(&self) -> bool {
        self.contains(Self::SHIELD)
    }

    #[inline]
    pub fn grab(&self) -> bool {
        self.contains(Self::GRAB)
    }

    #[inline]
    pub fn set_attack(&mut self, value: bool) {
        self.set_value(Self::ATTACK, value);
    }

    #[inline]
    pub fn set_special(&mut self, value: bool) {
        self.set_value(Self::SPECIAL, value);
    }

    #[inline]
    pub fn set_jump(&mut self, value: bool) {
        self.set_value(Self::JUMP, value);
    }

    #[inline]
    pub fn set_shield(&mut self, value: bool) {
        self.set_value(Self::SHIELD, value);
    }

    #[inline]
    pub fn set_grab(&mut self, value: bool) {
        self.set_value(Self::GRAB, value);
    }

    #[inline]
    fn set_value(&mut self, flags: Self, value: bool) {
        if value {
            self.insert(flags);
        } else {
            self.remove(flags);
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Axis1D(pub i8);

impl From<Axis1D> for f32 {
    fn from(value: Axis1D) -> Self {
        Self::from(value.0) / Self::from(i8::MAX)
    }
}

impl From<f32> for Axis1D {
    fn from(value: f32) -> Self {
        Self((value.clamp(-1.0, 1.0) * 127.0) as i8)
    }
}

impl fmt::Debug for Axis1D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f32::from(*self).fmt(f)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Axis2D {
    pub x: Axis1D,
    pub y: Axis1D,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerInputFrame {
    pub movement: Axis2D,
    pub smash: Axis2D,
    pub buttons: Buttons,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerInput {
    pub previous: PlayerInputFrame,
    pub current: PlayerInputFrame,
}

impl PlayerInput {
    pub fn tick(&mut self) {
        self.previous = self.current;
    }

    pub fn was_pressed(&self) -> Buttons {
        self.previous.buttons & !self.current.buttons
    }

    pub fn was_released(&self) -> Buttons {
        !self.previous.buttons & self.current.buttons
    }
}
