//! Renders a 2D scene containing a single, moving sprite.

use bevy::{prelude::*, utils::hashbrown::HashMap, window::PrimaryWindow};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, States)]
enum AppState {
    StartLevel(usize),
    Game,
}

fn main() {
    App::new()
        .insert_state(AppState::StartLevel(0))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, initial_setup)
        .add_systems(
            Update,
            (sprite_movement, text_update_system, collision_update_system)
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(
            Update,
            (sprite_draw, obstacle_draw).run_if(in_state(AppState::Game)),
        )
        .add_systems(
            Update,
            (sprite_draw, obstacle_draw).run_if(in_state(AppState::StartLevel(0))),
        )
        .add_systems(Update, (check_start_level,))
        .run();
}

// All objects part of the level need this component so they can be despawned
#[derive(Component)]
struct PartOfLevel;

#[derive(Component)]
struct PartOfStart;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct KillerObstacle;

#[derive(Component)]
struct Obstacle {
    pos: Vec2,
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

fn setup_obstacles(commands: &mut Commands, asset_server: Res<AssetServer>) {
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
        let height_of_wall = 200.0;
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

    // Load all sprites
    let all_assets = vec![
        "racecar_center.png",
        "racecar_left.png",
        "racecar_right.png",
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

fn setup_start(commands: &mut Commands, all_sprites: &AllSprite) {
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
            left: Val::Percent(40.0),
            ..default()
        }),
        PartOfStart,
    ));
}

fn setup_level(commands: &mut Commands, asset_server: Res<AssetServer>, all_sprites: &AllSprite) {
    let mut transform = Transform::from_xyz(100., 0., 0.);
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
        },
        PartOfLevel,
    ));
    setup_obstacles(commands, asset_server);

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
    mut sprite_position: Query<(&mut Car, &mut Transform, &mut Handle<Image>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    sprites: Query<&AllSprite>,
) {
    for (mut car, mut transform, mut texture) in &mut sprite_position {
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
        transform.translation.y = -200.0;
        transform.translation.x = car.pos.x;

        transform.rotation =
            Quat::from_rotation_z(car.direction.to_angle() - std::f32::consts::FRAC_PI_2);
    }
}

fn text_update_system(time: Res<Time>, mut query: Query<&mut Text, With<TimerText>>) {
    for mut text in &mut query {
        text.sections[0].value = format!("Time: {}", time.elapsed_seconds().floor());
    }
}

fn obstacle_draw(mut obstacles: Query<(&Obstacle, &mut Transform)>, car: Query<&Car>) {
    let car = car.iter().next().unwrap();
    for (obstacle, mut transform) in &mut obstacles {
        transform.translation.x = obstacle.pos.x;
        transform.translation.y = obstacle.pos.y - car.pos.y;
    }
}

fn collision_update_system(
    obstacles: Query<&Obstacle>,
    mut car: Query<&mut Car>,
    mut next_state: ResMut<NextState<AppState>>,
    to_delete: Query<Entity, With<PartOfLevel>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sprites: Query<&AllSprite>,
) {
    let car = car.single_mut();
    let mut game_over = false;
    for obstacle in &obstacles {
        if car.pos.distance(obstacle.pos) < 100. {
            // Game over
            game_over = true;
        }
    }

    if game_over {
        // delete things part of the level
        for entity in to_delete.iter() {
            commands.entity(entity).despawn();
        }
        setup_start(&mut commands, sprites.get_single().unwrap());
        setup_level(&mut commands, asset_server, sprites.get_single().unwrap());
        next_state.set(AppState::StartLevel(0));
    }
}

fn check_start_level(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut to_delete: Query<Entity, With<PartOfStart>>,
    mut commands: Commands,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        for entity in to_delete.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(AppState::Game);
    }
}
