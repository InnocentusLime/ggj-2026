use core::f32;

use super::prelude::*;

pub const PLAYER_ATTACK_DELAY: f32 = 1.0;

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
        AttackCooldown(0.0),
        LookAngle(0.0),
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

    for (_, (tf, control, look)) in world.query_mut::<(&Transform, &mut KinematicControl, &mut LookAngle)>().with::<&PlayerState>() {
        if do_walk {
            control.dr = walk_dir * dt * cfg.player.speed;
        } else {
            control.dr = Vec2::ZERO;
        }
        let mut look_dir = (input.aim - tf.pos).normalize_or(Vec2::X);
        // look_dir.y *= -1.0;
        look.0 = look_dir.to_angle();
        dump!("aim: {} pos: {}", input.aim, tf.pos);

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

    for (_, (attrs, cooldown)) in world.query_mut::<(&PlayerAttributes, &mut AttackCooldown)>().with::<&PlayerState>() {
        if !attrs.strong_against_grunts {
            continue;
        }
        cooldown.0 -= dt;
        if cooldown.0 > 0.0 {
            continue;
        }
        if input.attack_down {
            cooldown.0 = PLAYER_ATTACK_DELAY;
        }
    }
}

pub fn spawn_attack(world: &mut World, cmds: &mut CommandBuffer) {
    for (_, (tf, angle, cooldown)) in world.query_mut::<(&Transform, &LookAngle, &mut AttackCooldown)>().with::<&PlayerState>() {
        if cooldown.0 != PLAYER_ATTACK_DELAY {
            continue;
        }
        cmds.spawn((
            Transform {
                pos: tf.pos + 10.0 * Vec2::from_angle(angle.0),
                angle: angle.0,
            },
            Attack,
            col_query::Damage::new(
                Shape::Rect { 
                    width: 4.0, 
                    height: 10.0, 
                }, 
                col_group::GRUNT, 
                col_group::NONE,
            ),
            Lifetime(0.3),
        ));
    }
}

pub fn tick_attack(dt: f32, world: &mut World, cmds: &mut CommandBuffer) {
    for (ent, tick) in &mut world.query::<&mut Lifetime>().with::<&Attack>() {
        tick.0 -= dt;
        if tick.0 < 0.0 {
            cmds.despawn(ent);
        }
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
