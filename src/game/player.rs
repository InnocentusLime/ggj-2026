use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2, resources: &Resources) {
    let pos = if resources.is_start {
        pos
    } else {
        resources.player_pos
    };
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
        PlayerAttributes::default(),
        CurrentMask(resources.current_mask),
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

    for (_, control) in world.query_mut::<&mut KinematicControl>().with::<&PlayerState>() {
        if do_walk {
            control.dr = walk_dir * dt * cfg.player.speed;
        } else {
            control.dr = Vec2::ZERO;
        }

        if let Some(mask) = input.mask_request {
            info!("Mask queued: {mask}");
        }
    }

    for (_, (mask)) in world.query_mut::<(&mut CurrentMask)>().with::<&PlayerState>() {
        let Some(new_mask) = input.mask_request else {
            continue;
        };
        let Some(_) = resources.masks.get(new_mask as usize) else {
            continue;
        };
        if !resources.mask_unlock[new_mask as usize] {
            continue;
        }
        info!("new_mas: {new_mask}");
        mask.0 = new_mask;
    }
}

pub fn propagate_attriutes(world: &mut World, resources: &mut Resources) {
    for (_, (mask, attrs)) in world.query_mut::<(&CurrentMask, &mut PlayerAttributes)>().with::<&PlayerState>() {
        let Some(new_attrs) = resources.masks.get(mask.0 as usize) else {
            continue;
        };
        *attrs = *new_attrs;
        resources.current_mask = mask.0;
    }
}

pub fn publish_pos(world: &mut World, resources: &mut Resources) {
    for (_, tf) in world.query_mut::<&Transform>().with::<&PlayerState>() {
        resources.player_pos = tf.pos;
    }
}
