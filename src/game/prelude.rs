pub use super::components::*;
pub use hecs::{CommandBuffer, EntityBuilder, World};
pub use lib_game::*;
pub use macroquad::prelude::*;

pub const TILE_SIDE_F32: f32 = lib_asset::level::TILE_SIDE as f32;

pub fn find_player(world: &World) -> Option<Vec2> {
    for (_, (tf, hp)) in world.query::<(&Transform, &Health)>().with::<&PlayerState>().into_iter() {
        if hp.is_invulnerable {
            continue;
        }
        return Some(tf.pos);
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