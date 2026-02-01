use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, key: u32, pos: Vec2, resources: &Resources) {
    if resources.key_unlock[key as usize] {
        return;
    }
    builder.add_bundle((
        Transform::from_pos(pos),
        Key(key),
        col_query::Interaction::new(
            Shape::Circle { radius: 4.0 }, 
            col_group::PLAYER, 
            col_group::NONE,
        ),
    ));
}

pub fn pickup(world: &World, resources: &mut Resources, collisions: &CollisionSolver, cmds: &mut CommandBuffer) {
    for (ent, (col, key)) in &mut world.query::<(&col_query::Interaction, &Key)>().iter() {
        if collisions.collisions_for(col).is_empty() {
            continue;
        }
        cmds.despawn(ent);
        resources.key_unlock[key.0 as usize] = true;
    }
}