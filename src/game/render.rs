use hecs::World;
use super::prelude::*;

use crate::game::prelude::PlayerState;

pub fn render_player(world: &World, render: &mut Render) {
    for (_, (tf, health)) in &mut world.query::<(&Transform, &Health)>().with::<&PlayerState>().iter() {
        let mut tf = *tf;
        let mut player_color = WHITE;
        if should_flicker() && health.is_invulnerable {
            player_color.a = 0.5;
        }
        
        tf.pos -= Vec2::splat(TILE_SIDE_F32) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Mobs, 
            rect: atlas_tile(0, 0), 
            color: player_color, 
            sort_offset: 0.0, 
        });
    }
}

pub fn render_stabber(world: &World, render: &mut Render) {
    for (_, (tf, health)) in &mut world.query::<(&Transform, &Health)>().with::<&StabberState>().iter() {
        let mut tf = *tf;
        let mut grunt_color = WHITE;
        if should_flicker() && health.is_invulnerable {
            grunt_color.a = 0.5;
        }

        tf.pos -= Vec2::splat(TILE_SIDE_F32) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Mobs, 
            rect: atlas_tile(3, 4), 
            color: grunt_color, 
            sort_offset: 0.0, 
        });
    }
}

pub fn render_mask(world: &World, render: &mut Render) {
    for (_, (tf)) in &mut world.query::<(&Transform)>().with::<&Mask>().iter() {
        let mut tf = *tf;

        tf.pos -= Vec2::splat(TILE_SIDE_F32) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Items, 
            rect: atlas_tile(14, 1), 
            color: WHITE, 
            sort_offset: 0.0, 
        });
    }
}

pub fn render_hunter(world: &World, render: &mut Render) {
    for (_, (tf)) in &mut world.query::<(&Transform)>().with::<&Hunter>().iter() {
        let mut tf = *tf;

        tf.pos -= Vec2::splat(TILE_SIDE_F32) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Mobs, 
            rect: atlas_tile(2, 7), 
            color: WHITE, 
            sort_offset: 0.0, 
        });
    }
}

pub fn render_attack(world: &World, render: &mut Render) {
    for (_, (tf)) in &mut world.query::<(&Transform)>().with::<&Attack>().iter() {
        let mut tf = *tf;

        tf.pos -= (Vec2::splat(TILE_SIDE_F32) / 2.0).rotate(Vec2::from_angle(tf.angle));
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Mobs, 
            rect: atlas_tile(12, 1), 
            color: WHITE, 
            sort_offset: 0.0, 
        });
    }
}

fn should_flicker() -> bool {
    let flicker_rate = 10;
    let tick = get_time() as f32 * (flicker_rate as f32);
    tick.fract() <= 0.5
}