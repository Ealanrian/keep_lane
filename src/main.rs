use rand::prelude::*;
use rusty_engine::prelude::*;

const PLAYER_SPEED: f32 = 250.0;
const ROAD_SPEED: f32 = 400.0;

struct GameState {
    health_amount: u8,
    lost: bool,
    slowdown: f32,
    rotation: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            health_amount: 5,
            lost: false,
            slowdown: 0.0,
            rotation: 0.0,
        }
    }
}

fn main() {
    let mut game = Game::new();
    let mut window_descripter = WindowDescriptor::default();
    window_descripter.title = "road rage".to_string();

    game.window_dimensions.x = window_descripter.height;
    game.window_dimensions.y = window_descripter.width;
    println!("game dimensions{}", game.window_dimensions);
    game.window_settings(window_descripter);
    // game setup goes here
    create_road(&mut game);

    generate_obstacles(&mut game);

    create_player(&mut game);

    let healt_mesage = game.add_text("health_message", "Health: 5");
    healt_mesage.translation = Vec2::new(550.0, 320.0);

    //start music
    // game.audio_manager
    //     .play_music(MusicPreset::WhimsicalPopsicle, 0.2);

    game.add_logic(game_logic);
    game.run(GameState::default());
}

fn generate_obstacles(game: &mut Game<GameState>) {
    let obstacles_presets = vec![
        SpritePreset::RacingBarrelBlue,
        SpritePreset::RacingBarrelRed,
        SpritePreset::RacingConeStraight,
        SpritePreset::RollingBallRed,
        SpritePreset::RollingBallBlueAlt,
    ];
    for (i, preset) in obstacles_presets.into_iter().enumerate() {
        let obstacle = game.add_sprite(format!("obstacle{}", i), preset);
        obstacle.layer = 5.0;
        obstacle.collision = true;
        obstacle.scale = 0.5;
        obstacle.translation.x = thread_rng().gen_range(800.0..1600.0);
        obstacle.translation.y = thread_rng().gen_range(-300.0..300.0);
    }
}

fn create_player(game: &mut Game<GameState>) {
    //create player
    let player1 = game.add_sprite("player1", SpritePreset::RacingCarBlue);
    player1.translation.x = -500.0;
    player1.layer = 10.0;
    player1.collision = true;
}

fn create_road(game: &mut Game<GameState>) {
    // create the road;
    for i in 0..10 {
        let roadline = game.add_sprite(format!("roadline{}", i), SpritePreset::RacingBarrierWhite);
        roadline.scale = 0.1;
        roadline.translation.x = -600.0 + 150.0 * i as f32;
    }

    //21px
    let number_of_side_lines = (game.window_dimensions.y / 21.0) as i32;
    for i in 0..number_of_side_lines {
        let side_line_top =
            game.add_sprite(format!("topeline{}", i), SpritePreset::RacingBarrierWhite);
        side_line_top.scale = 0.1;
        side_line_top.translation.x = -600.0 + 21.0 * i as f32;
        side_line_top.translation.y += 90.0;
        let side_line_bottom =
            game.add_sprite(format!("bottemine{}", i), SpritePreset::RacingBarrierWhite);
        side_line_bottom.scale = 0.1;
        side_line_bottom.translation.x = -600.0 + 21.0 * i as f32;
        side_line_bottom.translation.y -= 90.0;
    }
}

fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    // game logic goes here
    let mut direction: f32 = 0.0;

    if game_state.health_amount == 0 {
        game_state.lost = true;
        //game over.
        let game_over = engine.add_text("game_over", "Game Over");
        game_over.font_size = 128.0;
        engine.audio_manager.stop_music();
        engine.audio_manager.play_sfx(SfxPreset::Jingle3, 0.5);
        return;
    }

    determine_direction(engine, &mut direction);

    player_movement(engine, game_state, direction);

    process_objects(engine, game_state);

    update_health(engine, game_state)
}

fn update_health(engine: &mut Engine, game_state: &mut GameState) {
    let health_message = engine.texts.get_mut("health_message").unwrap();
    for event in engine.collision_events.drain(..) {
        if !event.pair.either_contains("player1") || event.state.is_end() {
            continue;
        }
        if game_state.health_amount > 0 {
            game_state.health_amount -= 1;
            health_message.value = format!("Health: {}", game_state.health_amount);
            engine.audio_manager.play_sfx(SfxPreset::Impact3, 0.5);
        }
    }
}

fn process_objects(engine: &mut Engine, game_state: &mut GameState) {
    let sprites = engine.sprites.values_mut();
    for sprite in sprites {
        if sprite.label.starts_with("roadline") {
            move_road(engine.delta_f32, sprite, game_state);
        }
        if sprite.label.starts_with("obstacle") {
            move_obstacles(engine.delta_f32, sprite, game_state);
        }
    }
}

fn move_obstacles(delta: f32, sprite: &mut Sprite, game_state: &mut GameState) {
    sprite.translation.x -= (ROAD_SPEED + game_state.slowdown) * delta;
    if sprite.translation.x < -800.0 {
        sprite.translation.x = thread_rng().gen_range(800.0..1600.0);
        sprite.translation.y = thread_rng().gen_range(-300.0..300.0);
    }
}

fn move_road(delta: f32, sprite: &mut Sprite, game_state: &mut GameState) {
    sprite.translation.x -= (ROAD_SPEED + game_state.slowdown) * delta;
    if sprite.translation.x < -675.0 {
        sprite.translation.x += 1500.0;
    }
}

fn player_movement(engine: &mut Engine, game_state: &mut GameState, direction: f32) {
    let player1 = engine.sprites.get_mut("player1").unwrap();
    if player1.translation.y > 91.0 {
        game_state.rotation = game_state.rotation + thread_rng().gen_range(0.02..0.10);
    } else if player1.translation.y < -91.0 {
        game_state.rotation = game_state.rotation + thread_rng().gen_range(-0.10..0.00);
    } else {
        game_state.rotation = 0.0;
        game_state.slowdown = 0.0;
    }
    let direction_modifier: f32;
    if game_state.rotation != 0.0 {
        if thread_rng().gen_bool(1.0 / 250.0) {
            game_state.rotation = 0.0;
        }

        game_state.slowdown = -150.0;
        direction_modifier = thread_rng().gen_range(-1.0..1.0);
    } else {
        direction_modifier = 0.0;
    }

    player1.translation.y +=
        (direction + direction_modifier) * (PLAYER_SPEED + game_state.slowdown) * engine.delta_f32;

    player1.rotation = direction * (0.15 + game_state.rotation);

    if player1.translation.y > 360.0 || player1.translation.y < -360.0 {
        game_state.health_amount = 0;
    }
}

fn determine_direction(engine: &mut Engine, direction: &mut f32) {
    if engine.keyboard_state.pressed(KeyCode::Up) || engine.keyboard_state.pressed(KeyCode::W) {
        *direction += 1.0;
    }
    if engine.keyboard_state.pressed(KeyCode::Down) || engine.keyboard_state.pressed(KeyCode::S) {
        *direction -= 1.0;
    }
}
