use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2, resources: &Resources) {
    let cfg = &resources.cfg;
    builder.add_bundle((
        PlayerState::Idle,
        DamageCooldown::new(resources.cfg.player.hit_cooldown),
        KinematicControl::new_slide(col_group::LEVEL),
        Transform::from_pos(pos),
        Health::new(cfg.player.max_hp),
        BodyTag {
            groups: col_group::CHARACTERS.union(col_group::PLAYER),
            shape: cfg.player.shape,
        },
    ));
}

pub fn controls(dt: f32, input: &InputModel, world: &mut World, resources: &Resources) {
    let cfg = &resources.cfg;
    let mut walk_dir = Vec2::ZERO;
    let mut do_walk = false;
    if input.left_movement_down {
        walk_dir += vec2(-1.0, 0.0);
        do_walk = true;
    }
    if input.up_movement_down {
        walk_dir += vec2(0.0, -1.0);
        do_walk = true;
    }
    if input.right_movement_down {
        walk_dir += vec2(1.0, 0.0);
        do_walk = true;
    }
    if input.down_movement_down {
        walk_dir += vec2(0.0, 1.0);
        do_walk = true;
    }
    walk_dir = walk_dir.normalize_or_zero();

    for (_, control) in world.query_mut::<&mut KinematicControl>() {
        if do_walk {
            control.dr = walk_dir * dt * cfg.player.speed;
        } else {
            control.dr = Vec2::ZERO;
        }
    }
}
