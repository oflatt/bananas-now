//! Renders a 2D scene containing a single, moving sprite.

use bevy::{audio::Volume, prelude::*, utils::hashbrown::HashMap};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, States)]
enum AppState {
    EndLevel {
        level: usize,
        did_win: bool,
        time: usize,
    },
    StartLevel(usize),
    Game,
}

fn main() {
    let draw_level = (
        sprite_draw,
        obstacle_draw,
        customer_draw,
        projectile_draw,
        draw_num_ammo,
        draw_goals,
    );
    App::new()
        .insert_state(AppState::StartLevel(0))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, initial_setup)
        .add_systems(
            Update,
            (
                sprite_movement,
                text_update_system,
                collision_update_system,
                detect_shoot_system,
                projectile_update,
                detect_projectile_hit,
                check_in_goal,
            )
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(Update, draw_level.run_if(in_state(AppState::Game)))
        .add_systems(Update, draw_level.run_if(in_state(AppState::StartLevel(0))))
        .add_systems(Update, draw_level.run_if(run_if_in_end_level))
        .add_systems(
            Update,
            (check_start_level,).run_if(in_state(AppState::StartLevel(0))),
        )
        .add_systems(Update, (check_end_to_start,).run_if(run_if_in_end_level))
        .run();
}

fn run_if_in_end_level(state: Res<State<AppState>>) -> bool {
    matches!(state.get(), AppState::EndLevel { .. })
}

// All objects part of the level need this component so they can be despawned
#[derive(Component)]
struct PartOfLevel;

#[derive(Component)]
struct PartOfStart;

#[derive(Component)]
struct PartOfEndLevel;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct KillerObstacle;

#[derive(Component)]
struct Obstacle {
    pos: Vec2,
}

#[derive(Component)]
struct Goal {
    pos: Vec2,
    radius: f32,
}

#[derive(Component)]
struct Car {
    pos: Vec2,
    vel: Vec2, // Velocity is calculated
    direction: Vec2,
    base_acc: f32,
    top_speed: f32,
    steer_strength: f32,
    drift_strength: f32,
    projectile_speed: f32,
    ammo: HashMap<Merch, usize>,
    start_time: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Merch {
    Banana,
}

#[derive(Component, Clone)]
struct Customer {
    pos: Vec2,
    wants: Merch,
}

#[derive(Component)]
struct CustomerBubble {
    pos: Vec2,
}

#[derive(Component)]
struct Projectile {
    pos: Vec2,
    vel: Vec2,
    merch: Merch,
}

#[derive(Component)]
struct AllSprite {
    map: HashMap<String, Handle<Image>>,
}

fn lv2_turns() -> Vec<(usize, f32)> {
    // (how many blocks to render, x position of those blocks)
    vec![
        (10, 0.0),
        (10, 50.0),
        (10, 100.0),
        (10, 150.0),
        (10, 200.0),
        (10, 150.0),
        (10, 100.0),
        (10, 50.0),
        (5, 0.0),
        (5, -50.0),
        (5, -100.0),
        (5, -150.0),
        (5, -200.0),
        (5, -250.0),
        (5, -300.0),
        (5, -250.0),
        (5, -200.0),
        (5, -150.0),
        (5, -100.0),
        (5, -50.0),
        (10, 0.0),
        (10, 50.0),
        (10, 100.0),
        (10, 150.0),
        (10, 200.0),
        (10, 150.0),
        (10, 100.0),
        (10, 50.0),
        (5, 0.0),
        (5, -50.0),
        (10, -100.0),
        (10, -150.0),
        (10, -200.0),
        (10, -250.0),
        (10, -300.0),
        (10, -250.0),
        (10, -200.0),
        (10, -150.0),
        (5, -100.0),
        (5, -50.0),
        (10, 0.0),
        (10, 50.0),
        (10, 100.0),
        (10, 150.0),
        (10, 200.0),
        (10, 150.0),
        (10, 100.0),
        (10, 50.0),
        (5, 0.0),
        (5, -50.0),
        (10, -100.0),
        (10, -150.0),
        (10, -200.0),
        (10, -250.0),
        (10, -300.0),
        (10, -250.0),
        (10, -200.0),
        (10, -150.0),
        (5, -100.0),
        (5, -50.0),
        (10, 0.0),
        (10, 50.0),
        (10, 100.0),
        (10, 150.0),
        (10, 200.0),
        (10, 150.0),
        (10, 100.0),
    ]
}

fn lv1_ammo() -> HashMap<Merch, usize> {
    vec![(Merch::Banana, 10)].into_iter().collect()
}

fn lv1_customers() -> Vec<Customer> {
    vec![Customer {
        pos: Vec2::new(20., 500.),
        wants: Merch::Banana,
    }]
}

fn lv1_turns() -> Vec<(usize, f32, f32)> {
    // (how many blocks to render, x position of those blocks)
    vec![
        (10, 0.0, 0.0),
        (10, 0.0, 10.0),
        (10, 0.0, 20.0),
        (10, 0.0, 40.0),
        (10, 0.0, 80.0),
        (10, 0.0, 120.0),
        (10, 0.0, 160.0),
        (10, 0.0, 120.0),
        (20, 50.0, 0.0),
        (30, 100.0, 0.0),
        // (40, 150.0, 0.0),
        (20, 150.0, 0.0),
        (20, 200.0, -20.0),
        (20, 200.0, -40.0),
        (30, 150.0, -20.0),
        (20, 100.0, 0.0),
        (10, 50.0, 0.0),
        (5, 0.0, 0.0),
        (5, -50.0, 0.0),
        (10, -100.0, 0.0),
        (20, -150.0, 0.0),
        (30, -200.0, 0.0),
        (40, -250.0, 0.0),
        (50, -300.0, 0.0),
        (30, -250.0, 0.0),
        (20, -200.0, 0.0),
        (10, -150.0, 0.0),
        (5, -100.0, 0.0),
        (5, -50.0, 0.0),
        (10, 0.0, 0.0),
        (20, 50.0, 0.0),
        (30, 100.0, 0.0),
        (40, 150.0, 0.0),
        (50, 200.0, 0.0),
        (30, 150.0, 0.0),
        (20, 100.0, 0.0),
        (10, 50.0, 0.0),
        (5, 0.0, 0.0),
        (5, -50.0, 0.0),
        (10, -100.0, 0.0),
        (20, -150.0, 0.0),
        (30, -200.0, 0.0),
        (40, -250.0, 0.0),
        (50, -300.0, 0.0),
        (30, -250.0, 0.0),
        (20, -200.0, 0.0),
        (10, -150.0, 0.0),
        (5, -100.0, 0.0),
        (5, -50.0, 0.0),
        (10, 0.0, 0.0),
        (20, 50.0, 0.0),
        (30, 100.0, 0.0),
        (40, 150.0, 0.0),
        (50, 200.0, 0.0),
        (30, 150.0, 0.0),
        (20, 100.0, 0.0),
        (10, 50.0, 0.0),
        (5, 0.0, 0.0),
        (5, -50.0, 0.0),
        (10, -100.0, 0.0),
        (20, -150.0, 0.0),
        (30, -200.0, 0.0),
        (40, -250.0, 0.0),
        (50, -300.0, 0.0),
        (30, -250.0, 0.0),
        (20, -200.0, 0.0),
        (10, -150.0, 0.0),
        (5, -100.0, 0.0),
        (5, -50.0, 0.0),
        (10, 0.0, 0.0),
        (20, 50.0, 0.0),
        (30, 100.0, 0.0),
        (40, 150.0, 0.0),
        (50, 200.0, 0.0),
        (30, 150.0, 0.0),
        (20, 100.0, 0.0),
    ]
}

fn get_texture(all_sprites: &AllSprite, key: &str) -> Handle<Image> {
    all_sprites.map.get(key).unwrap().clone()
}

fn world_to_screen_x(x: f32, z: f32) -> f32 {
    x / (z * f32::sin(0.341) - 400.0 * -f32::cos(0.341)) * (400.0)
}
fn world_to_screen_y(x: f32, z: f32) -> f32 {
    ((z * f32::cos(0.341) - 400.0 * f32::sin(0.341)) / (z * f32::sin(0.341) - 400.0 * -f32::cos(0.341))) * (200.0 / 1.428)
}
fn world_to_screen_scale(x: f32, z: f32) -> f32 {
    400.0 / (z * f32::sin(0.341) - 400.0 * -f32::cos(0.341))
}

fn setup_obstacles(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let mut transform = Transform::from_xyz(0., 20., -1.0);
    transform.scale = Vec3::new(0.1, 0.1, 0.1);
    // place one cone
    /*commands.spawn((
        SpriteBundle {
            texture: asset_server.load("cone.png"),
            transform,
            ..default()
        },
        Obstacle {
            pos: Vec2::new(100., 0.),
        },
        KillerObstacle,
    ));*/

    let mut ypos = -100.0;
    let left_side = -400.0; // Offset from the center coord of cones
                            // place level obstacles
                            // for (kill_ypos, kill_xpos) in lv1_killcones() {
                            //     let mut transform = Transform::from_xyz(kill_xpos, kill_ypos, -1.0);
                            //     transform.scale = Vec3::new(0.2, 0.2, 0.2);
                            //     commands.spawn((
                            //         SpriteBundle {
                            //             texture: asset_server.load("cone.png"),
                            //             transform,
                            //             ..default()
                            //         },
                            //         Obstacle {
                            //             pos: Vec2::new(kill_xpos, kill_ypos),
                            //         },
                            //         KillerObstacle,
                            //     ));
                            // }

    for (num, xpos, more_offset) in lv1_turns() {
        let height_of_wall = 160.0;
        let mut transform = Transform::from_xyz(xpos, height_of_wall, -1.);
        transform.scale = Vec3::new(0.1, 0.1, 0.1);
        for _n in 0..num {
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("static-wall.png"),
                    transform,
                    ..default()
                },
                Obstacle {
                    pos: Vec2::new(xpos + left_side + more_offset, ypos),
                },
                PartOfLevel,
            ));
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("static-wall.png"),
                    transform,
                    ..default()
                },
                Obstacle {
                    pos: Vec2::new(xpos - left_side - more_offset, ypos),
                },
                PartOfLevel,
            ));

            ypos += height_of_wall;
        }
    }
}

fn initial_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let mut audio = AudioBundle {
        source: asset_server.load("game_music.ogg"),
        ..default()
    };
    audio.settings.paused = true;
    commands.spawn(audio);
    
    // Load all sprites
    let all_assets = vec![
        "racecar_center.png",
        "racecar_left.png",
        "racecar_right.png",
        "smoke1.png",
        "smoke2.png",
        "banana.png",
        "green-circle.png",
        "banana-car.png",
        "banana-speech.png",
    ];
    let mut all_sprites = AllSprite {
        map: Default::default(),
    };
    for asset in all_assets {
        all_sprites
            .map
            .insert(asset.to_string(), asset_server.load(asset));
    }
    setup_start(&mut commands, &all_sprites);
    setup_level(&mut commands, asset_server, &all_sprites);
    commands.spawn(all_sprites);
}

fn setup_endlevel(commands: &mut Commands, did_win: bool) {
    let text = if did_win {
        "You won! Press Space to play again"
    } else {
        "You lost! Press Space to play again"
    };

    // add a text component
    let mut transform = Transform::from_xyz(0., 0., 3.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            text,
            TextStyle {
                font_size: 50.0,
                color: Color::GOLD,
                ..Default::default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(20.0),
            ..default()
        }),
        PartOfEndLevel,
    ));
}

fn setup_start(commands: &mut Commands, _all_sprites: &AllSprite) {
    // make text "press space to start"
    let mut transform = Transform::from_xyz(0., 0., 3.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            "Press Space to Start",
            TextStyle {
                font_size: 50.0,
                color: Color::GOLD,
                ..Default::default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(20.0),
            ..default()
        }),
        PartOfStart,
    ));

    // make text that says "give 10 bananas to 10 customers!"
    let mut transform = Transform::from_xyz(0., 0., 3.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            "Give 10 bananas to 10 customers!",
            TextStyle {
                font_size: 50.0,
                color: Color::GOLD,
                ..Default::default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            left: Val::Percent(20.0),
            ..default()
        }),
        PartOfStart,
    ));
}

fn setup_customers(commands: &mut Commands, all_sprites: &AllSprite) {
    for customer in lv1_customers() {
        let mut transform = Transform::from_xyz(customer.pos.x, customer.pos.y, 1.0);
        transform.scale = Vec3::new(1.0, 1.0, 1.0)*0.15;
        commands.spawn((
            SpriteBundle {
                texture: get_texture(all_sprites, "banana-car.png"),
                transform,
                ..default()
            },
            customer.clone(),
            PartOfLevel,
        ));

        // spawn a bubble above the car
        let mut transform = Transform::from_xyz(customer.pos.x, customer.pos.y + 100., 3.0);
        transform.scale = Vec3::new(1.0, 1.0, 1.0)*0.15;
        let bubble_pos = Vec2::new(customer.pos.x, customer.pos.y + 100.);
        commands.spawn((
            SpriteBundle {
                texture: get_texture(all_sprites, "banana-speech.png"),
                transform,
                ..default()
            },
            CustomerBubble { pos: bubble_pos},
            PartOfLevel,
        ));
    }
}

#[derive(Component)]
struct AmmoUi {}

#[derive(Component)]
struct AmmoUiText {
    merch: Merch,
}

fn setup_car(commands: &mut Commands, all_sprites: &AllSprite) {
    let mut transform = Transform::from_xyz(0., 0., 0.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        SpriteBundle {
            texture: get_texture(all_sprites, "racecar_center.png"),
            transform,
            ..default()
        },
        Car {
            pos: Vec2::new(100., 0.),
            vel: Vec2::new(0., 0.),
            direction: Vec2::new(0., 1.),
            base_acc: 1.,
            top_speed: 40.,
            steer_strength: 0.0015,
            drift_strength: 0.08,
            projectile_speed: 100.0,
            ammo: lv1_ammo(),
            start_time: 0.0,
        },
        PartOfLevel,
    ));

    let mut transform = Transform::from_xyz(500.0, 300.0, 4.);
    transform.scale = Vec3::new(0.05, 0.05, 0.05);
    // spawn banana UI element
    commands.spawn((
        SpriteBundle {
            texture: get_texture(all_sprites, "banana.png"),
            transform,
            ..default()
        },
        AmmoUi {},
        PartOfLevel,
    ));

    let banana_text = TextBundle::from_section(
        "10",
        TextStyle {
            font_size: 50.0,
            color: Color::GREEN,
            ..Default::default()
        },
    )
    .with_text_justify(JustifyText::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(5.0),
        right: Val::Px(5.0),
        ..default()
    });

    // spawn banana text UI element
    commands.spawn((
        banana_text,
        AmmoUiText {
            merch: Merch::Banana,
        },
        PartOfLevel,
    ));
}

fn setup_goals(commands: &mut Commands, all_sprites: &AllSprite) {
    let mut transform = Transform::from_xyz(100., 10000., 2.0);
    transform.scale = Vec3::new(1.0, 1.0, 1.0);
    // green circle for goal
    commands.spawn((
        SpriteBundle {
            texture: get_texture(all_sprites, "green-circle.png"),
            ..default()
        },
        Goal {
            pos: Vec2::new(100., 10000.),
            radius: 100.,
        },
        PartOfLevel,
    ));
}

fn setup_level(commands: &mut Commands, asset_server: Res<AssetServer>, all_sprites: &AllSprite) {
    setup_customers(commands, all_sprites);
    setup_car(commands, all_sprites);
    setup_obstacles(commands, &asset_server);
    setup_goals(commands, all_sprites);

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            "hello\nbevy!",
            TextStyle {
                font_size: 50.0,
                color: Color::GOLD,
                ..Default::default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            bottom: Val::Px(5.0),
            ..default()
        }),
        TimerText,
        PartOfLevel,
    ));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(
    mut sprite_position: Query<(&mut Car, &mut Handle<Image>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    sprites: Query<&AllSprite>,
) {
    for (mut car, mut texture) in &mut sprite_position {
        // Finds the car
        if keyboard_input.pressed(KeyCode::KeyA) {
            // Steering speed depends on speed of the car.
            car.direction = car
                .direction
                .rotate(Vec2::from_angle(car.steer_strength * car.vel.length()));
            *texture = get_texture(sprites.get_single().unwrap(), "racecar_left.png");
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            car.direction = car
                .direction
                .rotate(Vec2::from_angle(-car.steer_strength * car.vel.length()));
            *texture = get_texture(sprites.get_single().unwrap(), "racecar_right.png");
        } else {
            *texture = get_texture(sprites.get_single().unwrap(), "racecar_center.png");
        }

        let mut car_velocity_update = car.direction * car.base_acc;
        if car.vel.length() > 0.000001 {
            car_velocity_update -=
                car.vel.angle_between(car.direction).abs() * car.vel * car.drift_strength;
        }

        car.vel += car_velocity_update;

        // Limit the length of the vector to car.top_speed
        if car.vel.length() > car.top_speed {
            car.vel = car.vel.normalize() * car.top_speed;
        }

        car.pos = car.pos + car.vel;

        /*
        TODO add accel changes.
        vel += accel * time.delta_seconds(); // Check if this works in direction we need
        pos += vel * time.delta_seconds();
        */
    }
}

fn sprite_draw(
    mut sprite_position: Query<(&mut Car, &mut Transform, &mut Handle<Image>)>,
    sprites: Query<&AllSprite>,
) {
    for (mut car, mut transform, mut texture) in &mut sprite_position {
        // Update sprite
        // transform.translation.y = -200.0;
        // transform.translation.x = car.pos.x;
        transform.translation.x = world_to_screen_x(car.pos.x, 0.0);
        transform.translation.y = world_to_screen_y(car.pos.x, 0.0);
        transform.translation.z = world_to_screen_scale(car.pos.x, 0.0);
        transform.scale = world_to_screen_scale(car.pos.x, 0.0) * Vec3::new(0.2, 0.2, 0.2);
        transform.rotation =
            Quat::from_rotation_z(car.direction.to_angle() - std::f32::consts::FRAC_PI_2);
    }
}

fn text_update_system(
    time: Res<Time>,
    mut query: Query<&mut Text, With<TimerText>>,
    car: Query<&Car>,
) {
    for mut text in &mut query {
        text.sections[0].value = format!(
            "Time: {}",
            ((time.elapsed_seconds() - car.single().start_time) * 100.0).floor() / 100.0
        );
    }
}

fn obstacle_draw(mut obstacles: Query<(&Obstacle, &mut Transform)>, car: Query<&Car>) {
    let car = car.iter().next().unwrap();
    for (obstacle, mut transform) in &mut obstacles {
        // transform.translation.x = obstacle.pos.x;
        // transform.translation.y = obstacle.pos.y - car.pos.y;
        transform.translation.x = world_to_screen_x(obstacle.pos.x, obstacle.pos.y - car.pos.y);
        transform.translation.y = world_to_screen_y(obstacle.pos.x, obstacle.pos.y - car.pos.y);
        transform.translation.z = world_to_screen_scale(obstacle.pos.x, obstacle.pos.y - car.pos.y);
        transform.scale = world_to_screen_scale(obstacle.pos.x, obstacle.pos.y - car.pos.y) * Vec3::new(0.1, 0.1, 0.1);
    }
}

fn collision_update_system(
    obstacles: Query<&Obstacle>,
    mut car: Query<&mut Car>,
    mut next_state: ResMut<NextState<AppState>>,
    mut comands: Commands,
    time: Res<Time>,
    audio: Query<&AudioSink>,
) {
    let mut car = car.single_mut();
    let mut game_over = false;
    for obstacle in &obstacles {
        if car.pos.distance(obstacle.pos) < 100. {
            // Game over
            game_over = true;
        }
    }

    if game_over {
        next_state.set(AppState::EndLevel {
            level: 0,
            did_win: false,
            time: (time.elapsed_seconds() * 1000.0) as usize - (car.start_time * 1000.0) as usize,
        });

        setup_endlevel(&mut comands, false);
    }
}

// ignore too many arguments
#[allow(clippy::too_many_arguments)]
fn check_end_to_start(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    to_delete: Query<Entity, With<PartOfEndLevel>>,
    to_delete2: Query<Entity, With<PartOfLevel>>,
    mut commands: Commands,
    mut car: Query<&mut Car>,
    asset_server: Res<AssetServer>,
    sprites: Query<&AllSprite>,
    audio: Query<&AudioSink>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in to_delete.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(AppState::StartLevel(0));
        // set car start time
        let mut car = car.single_mut();
        car.start_time = 0.0;

        // delete things part of the level
        for entity in to_delete.iter().chain(to_delete2.iter()) {
            commands.entity(entity).despawn();
        }
        setup_start(&mut commands, sprites.get_single().unwrap());
        setup_level(&mut commands, asset_server, sprites.get_single().unwrap());
        audio.get_single().unwrap().pause();
    }
}

fn check_start_level(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    to_delete: Query<Entity, With<PartOfStart>>,
    mut commands: Commands,
    audio: Query<&AudioSink>,
    mut car: Query<&mut Car>,
    time: Res<Time>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        audio.get_single().unwrap().play();
        for entity in to_delete.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(AppState::Game);
        // set car start time
        let mut car = car.single_mut();
        car.start_time = time.elapsed_seconds();
    }
}

fn customer_draw(mut customers: Query<(&Customer, &mut Transform)>, car: Query<&Car>, mut bubbles: Query<&mut CustomerBubble>) {
    let car = car.iter().next().unwrap();
    for (customer, mut transform) in &mut customers {
        transform.translation.x = customer.pos.x;
        transform.translation.y = customer.pos.y - car.pos.y;
    }

    for mut bubble in &mut bubbles {
        bubble.pos.y = bubble.pos.y - car.pos.y;
    }
}

fn projectile_update(mut projectiles: Query<&mut Projectile>) {
    for mut projectile in &mut projectiles {
        projectile.pos = projectile.pos + projectile.vel;
    }
}

fn projectile_draw(mut projectiles: Query<(&Projectile, &mut Transform)>, car: Query<&Car>) {
    let car = car.iter().next().unwrap();
    for (projectile, mut transform) in &mut projectiles {
        transform.translation.x = projectile.pos.x;
        transform.translation.y = projectile.pos.y - car.pos.y;
    }
}

fn detect_shoot_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    all_sprites: Query<&AllSprite>,
    mut car: Query<&mut Car>,
) {
    let mut car = car.single_mut();
    for keycode in [KeyCode::KeyK, KeyCode::KeyJ] {
        if car.ammo.get(&Merch::Banana).unwrap_or(&0) != &0 && keyboard_input.just_pressed(keycode)
        {
            let mut transform = Transform::from_xyz(car.pos.x, car.pos.y, 1.0);
            transform.scale = Vec3::new(0.04, 0.04, 0.04);
            let angle = if keycode == KeyCode::KeyK {
                -std::f32::consts::FRAC_PI_2
            } else {
                std::f32::consts::FRAC_PI_2
            };
            commands.spawn((
                SpriteBundle {
                    texture: get_texture(all_sprites.get_single().unwrap(), "banana.png"),
                    transform,
                    ..default()
                },
                Projectile {
                    pos: car.pos,
                    // rotate direction so it shoots from right if J is pressed
                    vel: car.direction.rotate(Vec2::from_angle(angle)) * car.projectile_speed,
                    merch: Merch::Banana,
                },
                PartOfLevel,
            ));
            let _map = car.ammo.get_mut(&Merch::Banana).map(|x| {
                if *x > 0 {
                    *x -= 1
                }
            });
        }
    }
}

fn detect_projectile_hit(
    mut commands: Commands,
    projectiles: Query<(Entity, &Projectile)>,
    customers: Query<(Entity, &Customer)>,
) {
    for (projectile_entity, projectile) in &mut projectiles.iter() {
        for (customer_entity, customer) in &mut customers.iter() {
            if projectile.pos.distance(customer.pos) < 100. && projectile.merch == customer.wants {
                commands.entity(projectile_entity).despawn();
                commands.entity(customer_entity).despawn();
            }
        }
    }
}

fn draw_num_ammo(mut ammo_ui_text: Query<(&AmmoUiText, &mut Text)>, car: Query<&Car>) {
    let car = car.iter().next().unwrap();

    for (ammo, mut text) in &mut ammo_ui_text {
        text.sections[0].value = format!("{}", car.ammo.get(&ammo.merch).unwrap_or(&0));
    }
}

fn check_in_goal(
    mut next_state: ResMut<NextState<AppState>>,
    car: Query<&Car>,
    goals: Query<&Goal>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let car = car.iter().next().unwrap();
    for goal in goals.iter() {
        if car.pos.distance(goal.pos) < goal.radius {
            next_state.set(AppState::EndLevel {
                level: 0,
                did_win: true,
                time: (time.elapsed_seconds() * 1000.0) as usize
                    - (car.start_time * 1000.0) as usize,
            });

            setup_endlevel(&mut commands, true);
            break;
        }
    }
}

fn draw_goals(mut goals: Query<(&Goal, &mut Transform)>, car: Query<&Car>) {
    let car = car.iter().next().unwrap();
    for (goal, mut transform) in &mut goals {
        transform.translation.x = goal.pos.x;
        transform.translation.y = goal.pos.y - car.pos.y;
    }
}
