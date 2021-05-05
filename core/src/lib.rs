#[macro_use]
extern crate bitflags;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct PlayerId(pub u8);

pub mod character;
pub mod input;
