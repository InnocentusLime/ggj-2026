use hecs::World;
use super::prelude::*;

use crate::game::prelude::PlayerState;


pub fn render_player(world: &World, render: &mut Render) {
    for (_, (tf, body)) in &mut world.query::<(&Transform, &BodyTag)>().with::<&PlayerState>().iter() {
        let Shape::Rect { width, height } = body.shape else {
            continue;
        };
        let rect = Rect { x: 0.0, y: 0.0, w: 16.0, h: 16.0 };
        let mut tf = *tf;
        tf.pos -= vec2(width, height) / 2.0;
        render.sprite_buffer.push(SpriteData { 
            layer: 1, 
            tf, 
            texture: TextureId::Mobs, 
            rect, 
            color: WHITE, 
            sort_offset: 0.0, 
        });
    }
}