use super::{on_match_update, player::PlayerMovement};
use crate::time::FrameTimer;
use bevy::{ecs::system::SystemParam, math::*, prelude::*};
use fc_core::{
    geo::*,
    input::PlayerInput,
    player::Facing,
    stage::{RespawnPoint, Surface},
};

const DELTA_TIME: f32 = 1.0 / 60.0;

bitflags! {
    pub struct PhysicsGroups : u16 {
        const PLAYER = 1 << 0;
        const STAGE = 1 << 1;
        const HITBOX = 1 << 2;
    }
}

#[derive(Clone, Debug)]
pub enum Location {
    Airborne(Vec2),
    Respawning {
        point: Entity,
        remaining_time: FrameTimer,
    },
    Surface {
        surface: Entity,
        position: f32,
    },
}

impl Location {
    pub fn is_grounded(&self) -> bool {
        match self {
            Self::Airborne(_) => false,
            Self::Respawning { .. } => true,
            Self::Surface { .. } => true,
        }
    }

    pub fn calculate_position(&self, ctx: &mut StageContext) -> Vec3 {
        match self {
            Self::Airborne(position) => position.extend(0.0),
            Self::Surface { surface, position } => ctx
                .surface(*surface)
                .as_segment()
                .world_position(*position)
                .extend(0.0),
            Self::Respawning { point, .. } => ctx.respawn_point(*point).position.extend(0.0),
        }
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::Airborne(Vec2::ZERO)
    }
}

#[derive(Debug, Default)]
pub struct Body {
    pub mass: f32,
    pub facing: Facing,
    pub location: Location,
    pub velocity: Vec2,
    pub gravity: f32,
    pub drag: f32,
    pub ecb: EnvironmentCollisionBox,
}

impl Body {
    pub fn advance_tick(&mut self, ctx: &mut StageContext) {
        match &mut self.location {
            Location::Surface { surface, position } => {
                if self.velocity.y != 0.0 {
                    self.become_airborne(ctx);
                    self.advance_tick(ctx);
                    return;
                }

                self.velocity.y = 0.0;
                let mut surf = ctx.surface(*surface);
                let delta_x = self.velocity.x * DELTA_TIME;
                let left = surf.left().point;
                let right = surf.right().point;
                let (pos_x, query) = match *position + delta_x {
                    x if x < left.x => (x, Some(left)),
                    x if x > right.x => (x, Some(right)),
                    x => (x, None),
                };
                *position = pos_x;

                if query.is_none() {
                    return;
                }
                let mut target = query.unwrap();
                loop {
                    let mut found = false;
                    for (entity, test) in ctx.surfaces.iter() {
                        if test.has_end(target) && entity != *surface {
                            found = true;
                            *surface = entity;
                            target = surf.other(target).unwrap().point;
                            surf = test;
                            break;
                        }
                    }
                    if !found {
                        *position = position.clamp(surf.left().point.x, surf.right().point.x);
                    }
                    if surf.contains_x(*position) {
                        break;
                    }
                }
            }
            Location::Airborne(ref mut position) => {
                let prior = *position;
                if self.velocity.abs() != Vec2::ZERO {
                    let speed = self.velocity.length();
                    self.velocity = self
                        .velocity
                        .clamp_length_max(speed - self.drag * DELTA_TIME);
                }
                self.velocity.y -= self.gravity * DELTA_TIME;
                *position += self.velocity * DELTA_TIME;

                // Check for grounded checks.
                let delta = LineSegment2D::new(prior, *position);
                if let Some(location) = ctx.collision_check(delta) {
                    self.velocity.y = 0.0;
                    self.location = location;
                }
            }
            Location::Respawning {
                point,
                remaining_time,
            } => {
                remaining_time.tick();
                if self.velocity != Vec2::ZERO && remaining_time.is_done() {
                    let mut respawn = ctx.respawn_point(*point);
                    self.location = Location::Airborne(respawn.position);
                    respawn.occupied_by = None;
                }
            }
        }
    }

    fn become_airborne(&mut self, ctx: &mut StageContext) {
        let position = self.location.calculate_position(ctx);
        self.location = Location::Airborne(position.xy());
    }
}

#[derive(Debug, Default, Clone)]
pub struct EnvironmentCollisionBox(pub Bounds2D);

impl EnvironmentCollisionBox {
    pub fn top(&self) -> Vec2 {
        self.0.center + Vec2::new(0.0, self.0.extents.y)
    }

    pub fn bottom(&self) -> Vec2 {
        self.0.center - Vec2::new(0.0, self.0.extents.y)
    }

    pub fn left(&self) -> Vec2 {
        self.0.center - Vec2::new(self.0.extents.x, 0.0)
    }

    pub fn right(&self) -> Vec2 {
        self.0.center + Vec2::new(self.0.extents.x, 0.0)
    }

    pub fn translate(&mut self, delta: Vec2) {
        self.0.center += delta;
    }

    pub fn segments(&self) -> [LineSegment2D; 4] {
        [
            LineSegment2D::new(self.bottom(), self.left()),
            LineSegment2D::new(self.left(), self.top()),
            LineSegment2D::new(self.top(), self.right()),
            LineSegment2D::new(self.right(), self.bottom()),
        ]
    }
}

impl From<EnvironmentCollisionBox> for Bounds2D {
    fn from(value: EnvironmentCollisionBox) -> Self {
        value.0.clone()
    }
}

#[derive(SystemParam)]
pub struct StageContext<'a> {
    pub surfaces: Query<'a, (Entity, &'static Surface)>,
    pub respawn_points: Query<'a, &'static mut RespawnPoint>,
}

impl<'a> StageContext<'a> {
    pub fn surface(&self, entity: Entity) -> &Surface {
        self.surfaces.get(entity).expect("Missing surface.").1
    }

    pub fn respawn_point(&mut self, entity: Entity) -> Mut<RespawnPoint> {
        self.respawn_points
            .get_mut(entity)
            .expect("Missing respawn point.")
    }

    /// Checks if a body's motion intersects with stage geometry.
    pub fn collision_check(&self, movement: LineSegment2D) -> Option<Location> {
        if movement.start.y < movement.end.y {
            return None;
        }

        for (entity, surface) in self.surfaces.iter() {
            let segment = surface.as_segment();
            if movement.intersects(segment) {
                return Some(Location::Surface {
                    surface: entity,
                    position: movement.end.x,
                });
            }
        }
        None
    }
}

fn move_players(mut players: Query<(&mut Body, &mut PlayerMovement, &PlayerInput)>) {
    for (mut body, mut movement, input) in players.iter_mut() {
        body.velocity.x = f32::from(input.current.movement.x) * 3.0;

        // Handle jumps
        if body.location.is_grounded() {
            movement.reset_jumps();
        }
        if input.was_pressed().jump() {
            if let Some(power) = movement.next_jump_power() {
                info!("Jumped {}", power);
                body.velocity.y = power;
            }
        }
    }
}

/// System to update existing bodies
fn update_bodies(mut stage: StageContext, mut bodies: Query<(&mut Body, &mut Transform)>) {
    for (mut body, mut transform) in bodies.iter_mut() {
        body.advance_tick(&mut stage);
        // Update visual positions
        transform.translation = body.location.calculate_position(&mut stage);
    }
}

pub(super) fn build(builder: &mut AppBuilder) {
    builder.add_system_set(
        on_match_update()
            .with_system(update_bodies.system())
            .with_system(move_players.system()),
    );
}
