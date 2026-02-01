use hecs::World;
use super::prelude::*;

use crate::game::prelude::PlayerState;

pub const FRACTION_COLOR: &[Color] = &[
    WHITE,
    RED,
    BLUE,
    GREEN,
];

pub const KEY_COLOR: &[Color] = &[
    YELLOW,
    PURPLE,
];

pub fn render_player(world: &World, render: &mut Render) {
    for (_, (tf, health, mask)) in &mut world.query::<(&Transform, &Health, &CurrentMask)>().with::<&PlayerState>().iter() {
        let mut tf = *tf;
        let mut player_color = FRACTION_COLOR[mask.0 as usize];
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
        let mut grunt_color = FRACTION_COLOR[1];
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
    for (_, (tf, mask)) in &mut world.query::<(&Transform, &Mask)>().iter() {
        let mut tf = *tf;

        tf.pos -= Vec2::splat(TILE_SIDE_F32) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Items, 
            rect: atlas_tile(14, 1), 
            color: FRACTION_COLOR[mask.0 as usize], 
            sort_offset: 0.0, 
        });
    }
}

pub fn render_key(world: &World, render: &mut Render) {
    for (_, (tf, key)) in &mut world.query::<(&Transform, &Key)>().iter() {
        let mut tf = *tf;

        let rect = if key.0 == 0 {
            atlas_tile(10, 4)
        } else {
            atlas_tile(11, 4)
        };

        tf.pos -= Vec2::splat(TILE_SIDE_F32) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Items, 
            rect, 
            color: KEY_COLOR[key.0 as usize], 
            sort_offset: 0.0, 
        });
    }
}

pub fn render_door(world: &World, render: &mut Render) {
    for (_, (tf, door)) in &mut world.query::<(&Transform, &Door)>().iter() {
        let mut tf = *tf;

        tf.pos -= Vec2::splat(TILE_SIDE_F32) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Objs, 
            rect: atlas_tile(7, 0),
            color: KEY_COLOR[door.0 as usize], 
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
            color: FRACTION_COLOR[2], 
            sort_offset: 0.0, 
        });
    }
}

pub fn render_attack(world: &World, render: &mut Render) {
    for (_, (tf, life)) in &mut world.query::<(&Transform, &Lifetime)>().with::<&Attack>().iter() {
        let mut tf = *tf;
        let a = ((life.0 / 0.3) - 0.7).exp();

        tf.pos -= (Vec2::splat(TILE_SIDE_F32) / 2.0).rotate(Vec2::from_angle(tf.angle));
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Mobs, 
            rect: atlas_tile(12, 1), 
            color: WHITE.with_alpha(a), 
            sort_offset: 0.0, 
        });
    }
}

fn should_flicker() -> bool {
    let flicker_rate = 10;
    let tick = get_time() as f32 * (flicker_rate as f32);
    tick.fract() <= 0.5
}