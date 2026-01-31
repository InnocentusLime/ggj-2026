use hecs::World;
use super::prelude::*;

use crate::game::prelude::PlayerState;

pub fn render_player(world: &World, render: &mut Render) {
    for (_, (tf, body, health)) in &mut world.query::<(&Transform, &BodyTag, &Health)>().with::<&PlayerState>().iter() {
        let Shape::Rect { width, height } = body.shape else {
            continue;
        };
        let mut tf = *tf;
        let mut player_color = WHITE;
        if should_flicker() && health.is_invulnerable {
            player_color.a = 0.5;
        }
        
        tf.pos -= vec2(width, height) / 2.0;
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
    for (_, (tf, body)) in &mut world.query::<(&Transform, &BodyTag)>().with::<&StabberState>().iter() {
        let Shape::Rect { width, height } = body.shape else {
            continue;
        };
        let mut tf = *tf;

        tf.pos -= vec2(width, height) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Mobs, 
            rect: atlas_tile(3, 4), 
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