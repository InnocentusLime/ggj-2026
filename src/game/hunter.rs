use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2, resources: &Resources) {
    let cfg = &resources.cfg;
    builder.add_bundle((
        Hunter,
        DamageCooldown::new(resources.cfg.stabber.hit_cooldown),
        KinematicControl::new_slide(col_group::LEVEL),
        Transform::from_pos(pos),
        Health::new(cfg.stabber.max_hp),
        BodyTag {
            groups: col_group::CHARACTERS.union(col_group::ENEMY),
            shape: cfg.stabber.shape,
        },
        col_query::Damage::new(
            cfg.stabber.stab_box, 
            col_group::PLAYER, 
            col_group::NONE,
        ),
    ));
}

pub fn think(dt: f32, world: &World, resources: &Resources) {
    let cfg = &resources.cfg;
    for (_, (tf, kin)) in &mut world.query::<(&Transform, &mut KinematicControl)>().with::<&Hunter>().iter() {
        let Some((player, attrs)) = find_player(world) else {
            kin.dr = Vec2::ZERO;
            continue;
        };
        if !attrs.invisible_to_grunts {
            kin.dr = Vec2::ZERO;
            continue;
        }
        let player_dir = (player - tf.pos).normalize_or_zero();
        kin.dr = player_dir * cfg.stabber.speed * dt;
    }
}