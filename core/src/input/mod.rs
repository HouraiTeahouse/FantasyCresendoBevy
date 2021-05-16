use bevy::math::{Vec2, Vec3};
use bevy_input::{gamepad::GamepadButton, keyboard::KeyCode, Input};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{collections::HashMap, hash::Hash, ops::{Add, Sub}};

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct Buttons: u8 {
        const ATTACK = 1 << 0;
        const SPECIAL = 1 << 1;
        const JUMP = 1 << 2;
        const SHIELD = 1 << 3;
        const GRAB = 1 << 4;
    }
}

impl Buttons {
    pub const ALL: &'static [Self] = &[
        Self::ATTACK,
        Self::SPECIAL,
        Self::JUMP,
        Self::SHIELD,
        Self::GRAB,
    ];

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
    pub fn set_value(&mut self, flags: Self, value: bool) {
        if value {
            self.insert(flags);
        } else {
            self.remove(flags);
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Axis1D(pub i8);

impl Add<Axis1D> for Axis1D {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        let x = std::cmp::min(i8::MAX as i16, (self.0 as i16) + (rhs.0 as i16));
        Self(x as i8)
    }
}

impl Sub<Axis1D> for Axis1D {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        let x = std::cmp::min(i8::MIN as i16, (self.0 as i16) - (rhs.0 as i16));
        Self(x as i8)
    }
}

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

impl Add<Axis2D> for Axis2D {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Axis2D> for Axis2D {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl From<Axis2D> for Vec2 {
    fn from(value: Axis2D) -> Self {
        Vec2::new(f32::from(value.x), f32::from(value.y))
    }
}

impl From<Axis2D> for Vec3 {
    fn from(value: Axis2D) -> Self {
        Vec3::from((Vec2::from(value), 0.0))
    }
}

impl From<Vec2> for Axis2D {
    fn from(value: Vec2) -> Self {
        Self {
            x: Axis1D::from(value[0]),
            y: Axis1D::from(value[1]),
        }
    }
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

    pub fn move_diff(&self) -> Axis2D {
        self.current.movement - self.previous.movement
    }

    pub fn smash_diff(&self) -> Axis2D {
        self.current.movement - self.previous.movement
    }
}

#[derive(Debug, Clone)]
pub enum InputSource {
    /// This player does not require a local input source. Their inputs may be sourced from
    /// external sources (i.e. a replay or the network)
    None,
    /// Reserved for when CPU players are available.
    CPU,
    /// The player is sourcing their inputs from the local keyboard.
    Keyboard {
        movement: ButtonAxis2D<KeyCode>,
        smash: ButtonAxis2D<KeyCode>,
        buttons: ButtonMapping<KeyCode>,
    },
    /// The player is sourcing their inputs from the a local gamepad.
    Gamepad {
        buttons: ButtonMapping<GamepadButton>,
    },
}

impl Default for InputSource {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonAxis1D<T> {
    pub pos: T,
    pub neg: T,
}

impl<T: Copy + Eq + Hash> ButtonAxis1D<T> {
    pub fn sample(&self, input: &Input<T>) -> Axis1D {
        Axis1D(match (input.pressed(self.pos), input.pressed(self.neg)) {
            (true, true) => 0_i8,
            (true, false) => i8::MAX,
            (false, true) => i8::MIN,
            (false, false) => 0_i8,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonAxis2D<T> {
    pub horizontal: ButtonAxis1D<T>,
    pub vertical: ButtonAxis1D<T>,
}

impl<T: Copy + Eq + Hash> ButtonAxis2D<T> {
    pub fn sample(&self, input: &Input<T>) -> Axis2D {
        Axis2D {
            x: self.horizontal.sample(input),
            y: self.vertical.sample(input),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ButtonMapping<T>(pub HashMap<Buttons, Vec<T>>);

impl<T: Copy + Eq + Hash> ButtonMapping<T> {
    pub fn evaluate_all(&self, input: &Input<T>) -> Buttons {
        let mut buttons = Buttons::empty();
        for button in Buttons::ALL {
            if self.evaluate(*button, input) {
                buttons.insert(*button);
            }
        }
        buttons
    }

    pub fn evaluate(&self, button: Buttons, input: &Input<T>) -> bool {
        if let Some(buttons) = self.0.get(&button) {
            for button in buttons.iter() {
                if input.pressed(*button) {
                    return true;
                }
            }
        }
        return false;
    }
}
