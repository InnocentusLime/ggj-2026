use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, key: u32, pos: Vec2, resources: &Resources) {
    if resources.door_open[key as usize] {
        return;
    }
    builder.add_bundle((
        Transform::from_pos(pos),
        Door(key),
        col_query::Interaction::new(
            Shape::Circle { radius: 10.0 }, 
            col_group::PLAYER, 
            col_group::NONE,
        ),
        BodyTag {
            groups: col_group::LEVEL,
            shape: Shape::Rect {
                width: TILE_SIDE_F32,
                height: TILE_SIDE_F32,
            },
        },
    ));
}

pub fn open(world: &World, resources: &mut Resources, collisions: &CollisionSolver, cmds: &mut CommandBuffer) {
    for (ent, (col, door)) in &mut world.query::<(&col_query::Interaction, &Door)>().iter() {
        if collisions.collisions_for(col).is_empty() {
            continue;
        }
        if !resources.key_unlock[door.0 as usize] {
            continue;
        }
        cmds.despawn(ent);
        resources.door_open[door.0 as usize] = true;
    }
}