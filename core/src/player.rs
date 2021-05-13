pub type PlayerId = u8;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Facing {
    Left = 0,
    Right = 1,
}

impl Facing {
    pub fn is_left(&self) -> bool {
        *self == Self::Left
    }

    pub fn is_right(&self) -> bool {
        *self == Self::Left
    }

    pub fn invert(&mut self) {
        *self = self.inverted()
    }

    pub fn inverted(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
