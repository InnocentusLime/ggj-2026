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