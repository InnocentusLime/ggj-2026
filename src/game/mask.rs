use macroquad::audio::{play_sound, PlaySoundParams};

use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2, resources: &Resources, mask_id: u32) {
    if resources.mask_unlock[mask_id as usize] {
        return;
    }
    builder.add_bundle((
        Transform::from_pos(pos),
        Mask(mask_id),
        col_query::Interaction::new(
            Shape::Circle { radius: 8.0 }, 
            col_group::PLAYER, 
            col_group::NONE,
        ),
    ));
}

pub fn pickup(world: &World, resources: &mut Resources, collisions: &CollisionSolver, cmds: &mut CommandBuffer) {
    for (ent, (col, mask)) in &mut world.query::<(&col_query::Interaction, &Mask)>().iter() {
        if collisions.collisions_for(col).is_empty() {
            continue;
        }
        cmds.despawn(ent);
        resources.mask_unlock[mask.0 as usize] = true;
        play_sound(&resources.sounds[&SoundId::Pickup], PlaySoundParams {
            looped: false,
            volume: 0.6,
        });
    }
}