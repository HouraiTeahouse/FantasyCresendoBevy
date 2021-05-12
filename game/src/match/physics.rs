bitflags! {
    pub struct PhysicsGroups : u16 {
        const PLAYER = 1 << 0;
        const STAGE = 1 << 1;
        const HITBOX = 1 << 2;
    }
}
