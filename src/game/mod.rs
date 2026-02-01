mod components;
mod player;
mod prelude;
mod render;
mod stabber;
mod mask;
mod hunter;
mod key;
mod door;

use lib_asset::level::*;
use lib_asset::FontId;
use prelude::*;

use crate::game::render::KEY_COLOR;

fn decide_next_state(world: &mut World) -> Option<AppState> {
    let player_dead = world
        .query_mut::<&Health>()
        .with::<&PlayerState>()
        .into_iter()
        .all(|(_, hp)| hp.value <= 0);

    if player_dead {
        return Some(AppState::GameOver);
    }

    None
}

async fn load_resources(resources: &mut Resources) {
    set_default_filter_mode(FilterMode::Nearest);

    resources.load_font(FontId::Quaver).await;
    resources.load_texture(TextureId::Mobs).await;
    resources.load_texture(TextureId::World).await;
    resources.load_texture(TextureId::Items).await;
    resources.load_texture(TextureId::Objs).await;
    resources.load_sound(SoundId::Hurt).await;
    resources.load_sound(SoundId::Open).await;
    resources.load_sound(SoundId::Pickup).await;
    resources.load_sound(SoundId::Swoosh).await;
    build_textures_atlas();
}

pub struct Project {
    do_ai: bool,
    do_player_controls: bool,
    transitions: Vec<fn(&mut World, &Resources)>,
    ais: Vec<fn(f32, &mut World, &Resources)>,
    anim_syncs: Vec<fn(&mut World, &Resources)>,
}

impl Project {
    pub async fn new(app: &mut App) -> Project {
        load_resources(&mut app.resources).await;

        let proj = Project {
            do_player_controls: true,
            do_ai: true,
            transitions: Vec::new(),
            ais: Vec::new(),
            anim_syncs: Vec::new(),
        };

        proj
    }
}

impl Game for Project {
    fn handle_command(&mut self, _app: &mut App, cmd: &DebugCommand) -> bool {
        match cmd.command.as_str() {
            "nopl" => self.do_player_controls = false,
            "pl" => self.do_player_controls = true,
            "noai" => self.do_ai = false,
            "ai" => self.do_ai = true,
            _ => return false,
        }
        true
    }

    fn debug_draws(&self) -> &[(&'static str, fn(&World, &Resources))] {
        &[("phys", draw_physics_debug), ("dmg", debug_damage_boxes)]
    }

    fn input_phase(
        &mut self,
        input: &lib_game::InputModel,
        dt: f32,
        resources: &mut lib_game::Resources,
        world: &mut World,
    ) {
        if self.do_player_controls {
            player::controls(dt, input, world, resources);
            player::propagate_attriutes(world, resources);
        }
        
        if self.do_ai {
            stabber::think(dt, world, resources);
            hunter::think(dt, world, resources);
        }

    }

    fn plan_collision_queries(
        &mut self,
        _dt: f32,
        resources: &lib_game::Resources,
        world: &mut World,
        cmds: &mut CommandBuffer,
    ) {
        if self.do_ai {
            for anim_sync in &self.anim_syncs {
                anim_sync(world, resources);
            }
        }
        player::spawn_attack(world, cmds, resources);
    }

    fn update(
        &mut self,
        dt: f32,
        resources: &mut lib_game::Resources,
        world: &mut World,
        collisions: &CollisionSolver,
        cmds: &mut CommandBuffer,
    ) -> Option<lib_game::AppState> {
        player::publish_data(world, resources);
        player::tick_attack(dt, world, cmds);
        mask::pickup(world, resources, collisions, cmds);
        key::pickup(world, resources, collisions, cmds);
        door::open(world, resources, collisions, cmds);
        stabber::do_drops(world, resources, cmds);
        decide_next_state(world)
    }

    fn render_export(
        &self,
        app_state: &AppState,
        _resources: &lib_game::Resources,
        world: &World,
        render: &mut Render,
    ) {
        if app_state.is_presentable() && *app_state != (AppState::Active { paused: true }) { 
            render::render_player(world, render);
            render::render_stabber(world, render);
            render::render_mask(world, render);
            render::render_hunter(world, render);
            render::render_attack(world, render);
            render::render_key(world, render);
            render::render_door(world, render);
        }
    }

    fn init_tile(
        &self,
        resources: &Resources,
        builder: &mut hecs::EntityBuilder,
        tile_x: u32,
        tile_y: u32,
        tile: Option<TileIdx>,
    ) {
        let tile_pos =
            vec2(tile_x as f32, tile_y as f32) * TILE_SIDE_F32 + Vec2::splat(TILE_SIDE_F32 / 2.0);
        let Some(tile) = tile else {
            builder.add(Transform::from_pos(tile_pos));
            builder.add(TileTy::Wall);
            return;
        };
        let ty = resources.level.map.tiles[&tile].ty;

        builder.add(Transform::from_pos(tile_pos));
        builder.add(ty);
        builder.add(tile);
        if ty == TileTy::Wall {
            builder.add(BodyTag {
                groups: col_group::LEVEL,
                shape: Shape::Rect {
                    width: TILE_SIDE_F32,
                    height: TILE_SIDE_F32,
                },
            });
        }
    }

    fn init_character(
        &self,
        resources: &Resources,
        builder: &mut hecs::EntityBuilder,
        def: CharacterDef,
    ) {
        match def.info {
            CharacterInfo::Player {} => player::init(builder, def.pos, resources),
            CharacterInfo::Stabber { drop_key } => stabber::init(builder, drop_key, def.pos, resources),
            CharacterInfo::Mask { id } => mask::init(builder, def.pos, resources, id),
            CharacterInfo::Hunter {} => hunter::init(builder, def.pos, resources),
            CharacterInfo::Door { key } => door::init(builder, key, def.pos, resources),
        }
    }

    fn draw_ui(&self, world: &World, resources: &Resources, app_state: AppState) {
        if !app_state.is_presentable() {
            draw_unpresentable_ui(true, app_state, resources);
            return;
        }

        if app_state == (AppState::Active { paused: true }) {
            draw_unpresentable_ui(false, app_state, resources);
            return;
        }

        let player_hp = player_health(world);
        let heart_params = DrawTextureParams {
            dest_size: None,
            source: Some(atlas_tile(14, 0)),
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };
        for hp_idx in 0..player_hp {
            draw_texture_ex(
                &resources.textures[&TextureId::Items], 
                -2.0 * TILE_SIDE_F32, 
                3.0 * TILE_SIDE_F32 + TILE_SIDE_F32 * hp_idx as f32, 
                WHITE, 
                heart_params.clone(),
            );
        }

        let player_mask = player_mask(world);
        let font = &resources.fonts[&FontId::Quaver];
        draw_text_ex(
            "MASK", 
            16.0 * TILE_SIDE_F32 + 1.0 * TILE_SIDE_F32, 
            3.0 * TILE_SIDE_F32, 
            TextParams { 
                font: Some(font), 
                font_size: 16, 
                font_scale: 1.0, 
                font_scale_aspect: 1.0,
                rotation: 0.0, 
                color: WHITE, 
            },
        );
        for mask_id in 0..resources.masks.len() {
            let color = if resources.mask_unlock[mask_id] {
                WHITE
            } else {
                DARKGRAY
            };
            let text = if player_mask == mask_id as u32 {
                format!("{}<", mask_id + 1)
            } else {
                format!("{}", mask_id + 1)
            };
            draw_text_ex(
                text,
                16.0 * TILE_SIDE_F32 + 1.0 * TILE_SIDE_F32, 
                4.0 * TILE_SIDE_F32 + TILE_SIDE_F32 * mask_id as f32, 
                TextParams { 
                    font: Some(font), 
                    font_size: 16, 
                    font_scale: 1.0, 
                    font_scale_aspect: 1.0,
                    rotation: 0.0, 
                    color, 
                },
            );
        }

        for idx in 0..2 {
            if !resources.key_unlock[idx] {
                continue;
            }
            
            let rect = if idx == 0 {
                atlas_tile(10, 4)
            } else {
                atlas_tile(11, 4)
            };

            draw_texture_ex(
                &resources.textures[&TextureId::Items], 
                16.0 * TILE_SIDE_F32 + 1.0 * TILE_SIDE_F32 + idx as f32 * TILE_SIDE_F32, 
                8.0 * TILE_SIDE_F32, 
                KEY_COLOR[idx],
                DrawTextureParams { 
                    dest_size: None,
                    source: Some(rect),
                    rotation: 0.0,
                    flip_x: false,
                    flip_y: false,
                    pivot: None,
                },
            );
        }
    }
}
            
fn draw_unpresentable_ui(hide: bool, app_state: AppState, resources: &Resources) {
    if hide {
        draw_rectangle(0.0, 0.0, 16.0 * TILE_SIDE_F32, 17.0 * TILE_SIDE_F32, BLACK);
    }

    let font = &resources.fonts[&FontId::Quaver];
    let params = TextParams { 
        font: Some(font), 
        font_size: 16, 
        font_scale: 1.0, 
        font_scale_aspect: 1.0,
        rotation: 0.0, 
        color: WHITE, 
    };
    let texts: &[&str] = match app_state {
        AppState::Start => &["Switch masks with number keys", "Escape through the purple door", "", "Press space to start"],
        AppState::GameOver => &["You died", "Press space to restart"],
        AppState::GameDone => &["You escaped"],
        AppState::Active { paused: true } => &["Paused"],
        _ => return,
    };
    for (id, text) in texts.iter().enumerate() { 
        let mut x = TILE_SIDE_F32 * 8.0;
        let y = TILE_SIDE_F32 * 8.0 + id as f32 * TILE_SIDE_F32 * 1.4;
        let measure = measure_text(
            text,
            params.font, 
            params.font_size,
            params.font_scale,
        );
        x -= (measure.width / 2.0);
        draw_text_ex(text, x, y, params.clone());
    }
}

fn debug_damage_boxes(world: &World, _resources: &Resources) {
    for (_, (tf, tag)) in &mut world.query::<(&Transform, &col_query::Damage)>() {
        draw_shape_lines(tf, &tag.collider, RED);
    }
}
