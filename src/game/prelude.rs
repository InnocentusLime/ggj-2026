pub use super::components::*;
use hecs::Entity;
pub use hecs::{CommandBuffer, EntityBuilder, World};
pub use lib_game::*;
pub use macroquad::prelude::*;

pub const VISION_CAST_SHAPE: Shape = Shape::Rect { width: 4.0, height: 4.0 };
pub const TILE_SIDE_F32: f32 = lib_asset::level::TILE_SIDE as f32;

pub fn new_vision_cast() -> VisionCast {
    VisionCast { 
        direction: Vec2::X, 
        shape: VISION_CAST_SHAPE, 
        found: None, 
        group: col_group::PLAYER.union(col_group::LEVEL),
    }
}

pub fn sees_player(vision: &VisionCast, player: Entity) -> bool {
    vision.found == Some(player)
}

pub fn find_player(world: &World) -> Option<(Entity, Vec2, PlayerAttributes)> {
    for (ent, (tf, attrs, hp)) in world.query::<(&Transform, &PlayerAttributes, &Health)>().with::<&PlayerState>().into_iter() {
        if hp.is_invulnerable {
            continue;
        }
        return Some((ent, tf.pos, *attrs));
    }
    None
}

pub fn player_health(world: &World) -> u32 {
    for (_, hp) in world.query::<&Health>().with::<&PlayerState>().into_iter() {
        return if hp.value < 0 { 0 } else { hp.value as u32 };
    }
    0
}

pub fn atlas_tile(x: u32, y: u32) -> Rect {
    Rect {
        x: x as f32 * TILE_SIDE_F32,
        y: y as f32 * TILE_SIDE_F32,
        w: TILE_SIDE_F32,
        h: TILE_SIDE_F32,
    }
}

pub fn player_mask(world: &World) -> u32 {
    for (_, mask) in world.query::<&CurrentMask>().with::<&PlayerState>().into_iter() {
        return mask.0
    }
    0
}