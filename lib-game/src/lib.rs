mod collisions;
mod components;
mod health;
mod input;
mod level_utils;
mod render;

#[cfg(feature = "dbg")]
pub mod dbg;

pub mod sys;

use hashbrown::{HashMap, HashSet};
use hecs::{Entity, EntityBuilder};

pub use collisions::*;
pub use components::*;
pub use input::*;
pub use level_utils::*;
pub use lib_asset::level::*;
pub use lib_asset::*;
pub use render::*;

#[macro_export]
#[cfg(feature = "dbg")]
macro_rules! dump {
    ($($arg:tt)+) => {
        $crate::dbg::GLOBAL_DUMP.put_line(std::format_args!($($arg)+));
    };
}

#[macro_export]
#[cfg(not(feature = "dbg"))]
macro_rules! dump {
    ($($arg:tt)+) => {
        /* NOOP */
    };
}

#[derive(Debug)]
pub struct DebugCommand {
    pub command: String,
    pub args: Vec<String>,
}

use hecs::{CommandBuffer, World};
use macroquad::{audio::Sound, prelude::*};

const GAME_TICKRATE: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Start,
    Active { paused: bool },
    GameOver,
    Win,
    GameDone,
    PleaseRotate,
    DebugFreeze,
}

#[derive(Debug)]
pub enum NextState {
    Load(String),
    AppState(AppState),
}

/// The trait containing all callbacks for the game,
/// that is run inside the App. It is usually best to
/// only keep configuration stuff inside this struct.
///
/// The application loop is structured as follows:
/// 1. Clearing the physics state
/// 2. Game::input_phase
/// 3. Physics simulation step and writeback
/// 4. Game::pre_physics_query_phase
/// 5. Handling of the physics queries
/// 6. Game::update
/// 7. Game::render
pub trait Game: 'static {
    fn handle_command(&mut self, app: &mut App, cmd: &DebugCommand) -> bool;

    /// Return the list of the debug draws. Debug draws are batches
    /// of (usually, macroquad) draw calls to assist you at debugging
    /// the game logic.
    ///
    /// These debug draws can be used in `dde` and `ddd` and will
    /// show up in `ddl`
    fn debug_draws(&self) -> &[(&'static str, fn(&World, &Resources))];

    /// Handle the user input. You also get the delta-time.
    fn input_phase(
        &mut self,
        input: &InputModel,
        dt: f32,
        resources: &mut Resources,
        world: &mut World,
    );

    /// Set up all physics queries. This can be considered as a sort of
    /// pre-update phase.
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn plan_collision_queries(
        &mut self,
        dt: f32,
        resources: &Resources,
        world: &mut World,
        cmds: &mut CommandBuffer,
    );

    /// Main update routine. You can request the App to transition
    /// into a new state by returning [Option::Some].
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn update(
        &mut self,
        dt: f32,
        resources: &mut Resources,
        world: &mut World,
        collisions: &CollisionSolver,
        cmds: &mut CommandBuffer,
    ) -> Option<AppState>;

    /// Export the game world for rendering.
    fn render_export(
        &self,
        state: &AppState,
        resources: &Resources,
        world: &World,
        render: &mut Render,
    );

    fn draw_ui(&self, world: &World, resources: &Resources, app_state: AppState);

    fn init_tile(
        &self,
        resources: &Resources,
        builder: &mut EntityBuilder,
        tile_x: u32,
        tile_y: u32,
        tile: Option<TileIdx>,
    );

    fn init_character(&self, resources: &Resources, builder: &mut EntityBuilder, def: CharacterDef);
}

impl AppState {
    /// Gives a hint whether the user should start
    /// rendering the game state or not
    pub fn is_presentable(&self) -> bool {
        matches!(
            self,
            AppState::Active { .. } | AppState::Win | AppState::DebugFreeze
        )
    }
}

/// The app run all the boilerplate code to make the game tick.
/// The following features are provided:
/// * State transitions and handling
/// * Debugging
/// * Physics handling
/// * Consistent tickrate timing
/// * Sound playing
/// * Integration with log-rs
/// * Drawing of the `dump!` macro
pub struct App {
    fullscreen: bool,
    old_size: (u32, u32),

    pub state: AppState,
    pub queued_level: Option<LevelId>,
    pub resources: Resources,
    accumelated_time: f32,

    camera: Camera2D,
    pub render: Render,
    col_solver: CollisionSolver,
    pub world: World,
    room_switches: [Entity; 4],
    cmds: CommandBuffer,

    render_world: bool,
    #[allow(unused)]
    freeze: bool,
}

impl App {
    pub async fn new(conf: &Conf) -> anyhow::Result<Self> {
        let mut resources = Resources::new();
        resources.cfg = load_game_config(&resources.resolver).await?;
        resources.masks = load_masks(&resources.resolver).await?; 
        resources.mask_unlock = vec![
            false; resources.masks.len()
        ];
        resources.mask_unlock[0] = true;

        Ok(Self {
            fullscreen: conf.fullscreen,
            old_size: (conf.window_width as u32, conf.window_height as u32),

            state: AppState::Start,
            queued_level: None,
            resources,
            accumelated_time: 0.0,

            camera: Camera2D::default(),
            render: Render::new(),
            col_solver: CollisionSolver::new(),
            world: World::new(),
            room_switches: [Entity::DANGLING; 4],
            cmds: CommandBuffer::new(),

            render_world: true,
            freeze: false,
        })
    }

    /// Just runs the game. This is what you call after loading all the resources.
    /// This method will run forever as it provides the application loop.
    pub async fn run<G: Game>(mut self, game: &mut G) {
        #[cfg(feature = "dbg")]
        let mut debug = dbg::DebugStuff::new(game);

        sys::done_loading();

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        loop {
            #[cfg(feature = "dbg")]
            debug.ui(&mut self, game);

            let input = InputModel::capture(&self.camera);
            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);
            self.fullscreen_toggles(&input);

            self.next_state(&input);
            if let Some(queued_level) = self.queued_level.take() {
                self.load_level(game, queued_level).await;
                self.state = AppState::Active { paused: false };
            }

            if do_tick {
                #[cfg(feature = "dbg")]
                debug.new_update();
                dump!("game state: {:?}", self.state);
                if matches!(self.state, AppState::Active { paused: false })
                    && let Some(next_state) = self.game_update(&input, game)
                {
                    self.state = next_state;
                }
            }

            self.update_camera();
            self.render.new_frame();
            self.render.buffer_tiles(&mut self.world);
            game.render_export(&self.state, &self.resources, &self.world, &mut self.render);
            self.render.render(
                &self.resources, 
                &self.camera, 
                self.render_world, 
                |_cam| game.draw_ui(&self.world, &self.resources, self.state),
                #[cfg(not(feature = "dbg"))]
                || {},
                #[cfg(feature = "dbg")]
                || debug.debug_draw(&self.world, &self.resources),
            );

            #[cfg(feature = "dbg")]
            debug.draw(&mut self);

            next_frame().await
        }
    }

    async fn load_level<G: Game>(&mut self, game: &mut G, level_id: LevelId) {
        info!("Loading level");
        let level = self
            .resources
            .resolver
            .load::<LevelDef>(level_id)
            .await
            .unwrap();
        self.render.set_atlas(
            &self.resources,
            level.map.atlas,
            level.map.atlas_margin,
            level.map.atlas_spacing,
        );

        self.world.clear();

        let tile_side = TILE_SIDE as f32;
        let level_width = (level.map.width * TILE_SIDE) as f32;
        let level_height = (level.map.height * TILE_SIDE) as f32;
        let mut spawn_interraction = |pos, width, height| self.world.spawn((
            Transform::from_pos(pos),
            col_query::Interaction::new(
                Shape::Rect { width, height }, 
                col_group::PLAYER, 
                col_group::NONE,
            ),
        )); 
        
        let left = spawn_interraction(
            vec2(0.0, level_height / 2.0),
            tile_side,
            level_height,
        );
        let top = spawn_interraction(
            vec2(level_width / 2.0, 0.0),
            level_width,
            tile_side,
        );
        let right = spawn_interraction(
            vec2(level_width, level_height / 2.0),
            tile_side,
            level_height,
        );
        let bot = spawn_interraction(
            vec2(level_width / 2.0, level_height),
            level_width,
            tile_side,
        );
        self.room_switches = [left, top, right, bot];

        // left
        if level_id.0.x < self.resources.level_id.0.x {
            self.resources.player_pos.x = level_width - 2.0 * tile_side; 
        }

        // top
        if level_id.0.y < self.resources.level_id.0.y {
            self.resources.player_pos.y = level_height - 2.0 * tile_side; 
        }

        // right
        if level_id.0.x > self.resources.level_id.0.x {
            self.resources.player_pos.x = 2.0 * tile_side; 
        }

        // bot
        if level_id.0.y > self.resources.level_id.0.y {
            self.resources.player_pos.y = 2.0 * tile_side; 
        }

        self.resources.level = level;
        self.resources.level_id = level_id;
        self.spawn_tiles(game);
        self.spawn_characters(game);
        self.resources.is_start = false;
    }

    fn spawn_tiles<G: Game>(&mut self, game: &G) {
        let level = &self.resources.level;
        for x in 0..level.map.width {
            for y in 0..level.map.height {
                let mut builder = EntityBuilder::new();
                let tile = level.map.tilemap[(x + y * level.map.width) as usize];
                game.init_tile(&self.resources, &mut builder, x, y, tile);
                self.world.spawn(builder.build());
            }
        }
    }

    fn spawn_characters<G: Game>(&mut self, game: &G) {
        for def in self.resources.level.characters.iter() {
            if self.resources.kill_memory.contains(&(self.resources.level_id, def.local_id)) {
                continue;
            }
            let mut builder = EntityBuilder::new();
            builder.add(ObjId(def.local_id));
            game.init_character(&self.resources, &mut builder, *def);
            self.world.spawn(builder.build());
        }
    }

    fn game_update<G: Game>(&mut self, input: &InputModel, game: &mut G) -> Option<AppState> {
        game.input_phase(input, GAME_TICKRATE, &mut self.resources, &mut self.world);

        health::reset(&mut self.world);
        health::update_cooldown(GAME_TICKRATE, &mut self.world);

        self.col_solver.import_colliders(&mut self.world);
        self.col_solver.export_kinematic_moves(&mut self.world);

        game.plan_collision_queries(
            GAME_TICKRATE,
            &self.resources,
            &mut self.world,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        self.col_solver.compute_collisions(&mut self.world);

        health::collect_damage(&mut self.world, &self.col_solver, &self.resources);
        health::apply_damage(&mut self.world);
        health::apply_cooldown(&mut self.world);
        health::despawn_on_zero_health(&mut self.world, &mut self.cmds, &mut self.resources);
        self.do_room_switch();

        let new_state = game.update(
            GAME_TICKRATE,
            &mut self.resources,
            &mut self.world,
            &self.col_solver,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        self.world.flush();

        new_state
    }

    fn do_room_switch(&mut self) {
        let check_level_switch = |switch| {
            let Ok(q) = self.world.get::<&col_query::Interaction>(switch) else {
                return false;
            };
            return !self.col_solver.collisions_for(&q).is_empty();
        };

        // left
        if check_level_switch(self.room_switches[0]) {
            let mut level_id = self.resources.level_id;
            if level_id.0.x == 0 {
                self.state = AppState::GameDone;
                return;
            } else {
                level_id.0.x -= 1;
            }
            self.queued_level = Some(level_id);
        }

        // top
        if check_level_switch(self.room_switches[1]) {
            let mut level_id = self.resources.level_id;
            level_id.0.y -= 1;
            self.queued_level = Some(level_id);
        }
        
        // right
        if check_level_switch(self.room_switches[2]) {
            let mut level_id = self.resources.level_id;
            level_id.0.x += 1;
            self.queued_level = Some(level_id);
        }

        // bot
        if check_level_switch(self.room_switches[3]) {
            let mut level_id = self.resources.level_id;
            level_id.0.y += 1;
            self.queued_level = Some(level_id);
        }
    }

    fn fullscreen_toggles(&mut self, input: &InputModel) {
        if !input.fullscreen_toggle_requested {
            return;
        }

        // NOTE: macroquad does not update window config when it goes fullscreen
        set_fullscreen(!self.fullscreen);

        if self.fullscreen {
            miniquad::window::set_window_size(self.old_size.0, self.old_size.1);
        }

        self.fullscreen = !self.fullscreen;
    }

    fn update_ticking(&mut self, real_dt: f32) -> bool {
        self.accumelated_time += real_dt;
        let lag_ms = (self.accumelated_time - 2.0 * GAME_TICKRATE) * 1000.0;
        if lag_ms > 1.0 {
            warn!("LAG by {lag_ms:.2}ms");
            self.accumelated_time = 0.0;
            false
        } else if self.accumelated_time >= GAME_TICKRATE {
            self.accumelated_time -= GAME_TICKRATE;
            true
        } else {
            false
        }
    }

    fn next_state(&mut self, input: &InputModel) {
        if self.state == AppState::DebugFreeze {
            return;
        }

        /* Normal state transitions */
        match self.state {
            AppState::GameOver if input.confirmation_detected => {
                self.state = AppState::Active { paused: false };
                self.queued_level = Some(LevelId(uvec2(0, 0)));
            }
            AppState::Win if input.confirmation_detected => {
                self.state = AppState::GameDone;
            }
            AppState::Start if input.confirmation_detected => {
                self.state = AppState::Active { paused: false };
                self.queued_level = Some(LevelId(uvec2(0, 0)));
            }
            AppState::Active { paused } if input.pause_requested => {
                self.state = AppState::Active { paused: !paused };
            }
            _ => (),
        }
    }

    fn update_camera(&mut self) {
        dump!("render: {}", self.camera.render_target.is_some());

        let view_height = 17.0 * TILE_SIDE as f32;
        let view_width = ((screen_width() / screen_height()) * view_height).floor();
        let new_cam = Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: view_width,
            h: view_height,
        });
        if new_cam.zoom == self.camera.zoom && new_cam.target == self.camera.target && self.camera.render_target.is_some() {
            return;
        }
        if self.camera.render_target.is_some() {
            return;
        }

        let target = render_target(
            view_width as u32 * 2,
            view_height as u32 * 2,
        );
        target.texture.set_filter(FilterMode::Nearest);
        self.resources.textures.insert(
            TextureId::Screen, 
            target.texture.clone(),
        );
        
        self.camera = new_cam;
        self.camera.render_target = Some(target);
        self.camera.zoom.y *= -1.0;

        // FIXME: magic numbers!
        self.camera.target = vec2(
            (0.5 * TILE_SIDE as f32) * 16.0,
            (0.5 * TILE_SIDE as f32) * 17.0,
        );
    }
}

pub struct Resources {
    pub cfg: GameCfg,
    pub resolver: FsResolver,
    pub is_start: bool,
    pub level_id: LevelId,
    pub level: LevelDef,
    pub player_pos: Vec2,
    pub textures: HashMap<TextureId, Texture2D>,
    pub fonts: HashMap<FontId, Font>,
    pub sounds: HashMap<SoundId, Sound>,
    pub masks: Vec<PlayerAttributes>,
    pub mask_unlock: Vec<bool>,
    pub key_unlock: [bool; 2],
    pub door_open: [bool; 2],
    pub current_mask: u32,
    pub kill_memory: HashSet<(LevelId, u32)>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            cfg: GameCfg::default(),
            is_start: true,
            resolver: FsResolver::new(),
            level_id: LevelId(uvec2(0, 0)),
            player_pos: vec2(0.0, 0.0),
            level: LevelDef::default(),
            textures: HashMap::new(),
            fonts: HashMap::new(),
            sounds: HashMap::new(),
            masks: Vec::new(),
            mask_unlock: Vec::new(),
            key_unlock: [false; 2],
            door_open: [false; 2],
            current_mask: 0,
            kill_memory: HashSet::new(),
        }
    }

    pub async fn load_sound(&mut self, sound_id: SoundId) {
        let sound = self.resolver.load(sound_id).await.unwrap();
        self.sounds.insert(sound_id, sound);
    }

    pub async fn load_texture(&mut self, texture_id: TextureId) {
        let texture = self.resolver.load(texture_id).await.unwrap();
        self.textures.insert(texture_id, texture);
    }

    pub async fn load_font(&mut self, font_id: FontId) {
        let font = self.resolver.load(font_id).await.unwrap();
        self.fonts.insert(font_id, font);
    }
}

impl Default for Resources {
    fn default() -> Self {
        Resources::new()
    }
}
