use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, drop_key: u32, pos: Vec2, resources: &Resources) {
    let cfg = &resources.cfg;
    builder.add_bundle((
        StabberState::Active,
        DamageCooldown::new(resources.cfg.stabber.hit_cooldown),
        KinematicControl::new_slide(col_group::LEVEL),
        Transform::from_pos(pos),
        Health::new(cfg.stabber.max_hp),
        BodyTag {
            groups: col_group::CHARACTERS.union(col_group::ENEMY).union(col_group::GRUNT),
            shape: cfg.stabber.shape,
        },
        col_query::Damage::new(
            cfg.stabber.stab_box, 
            col_group::PLAYER, 
            col_group::NONE,
        ),
        new_vision_cast(),
        Dropkey(drop_key),
    ));
}

pub fn do_drops(world: &World, resources: &Resources, cmds: &mut CommandBuffer) {
    for (_, (hp, tf, drop)) in &mut world.query::<(&Health, &Transform, &Dropkey)>().with::<&StabberState>() {
        if hp.value > 0 {
            continue;
        }
        if drop.0 == 0 {
            continue;
        }
        let mut builder = EntityBuilder::new();
        super::key::init(&mut builder, drop.0 - 1, tf.pos, resources);
        cmds.spawn(builder.build());
    }
}

pub fn think(dt: f32, world: &World, resources: &Resources) {
    let cfg = &resources.cfg;
    for (_, (tf, kin, vision)) in &mut world.query::<(&Transform, &mut KinematicControl, &mut VisionCast)>().with::<&StabberState>().iter() {
        let Some((player, player_pos, attrs)) = find_player(world) else {
            kin.dr = Vec2::ZERO;
            continue;
        };
        let player_dir = (player_pos - tf.pos).normalize_or_zero();
        vision.direction = player_dir;
        if attrs.invisible_to_grunts || !sees_player(vision, player) {
            kin.dr = Vec2::ZERO;
            continue;
        }
        let mut speed = cfg.stabber.speed;
        if attrs.strong_against_grunts {
            speed /= 3.0;
        }
        kin.dr = player_dir * speed * dt;
    }
}