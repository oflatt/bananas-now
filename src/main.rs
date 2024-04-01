//! Renders a 2D scene containing a single, moving sprite.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    utils::hashbrown::HashMap,
};

const HEIGHT_OF_WALL: f32 = 160.0;

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
        draw_objects,
        draw_num_ammo,
        fps_text_update_system,
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
        .add_plugins(FrameTimeDiagnosticsPlugin)
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

#[derive(Component, Clone)]
struct GameObject {
    pos: Vec2,
    draw_scale: f32,
}

#[derive(Component)]
struct KillerObstacle;

#[derive(Component)]
struct Obstacle;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct Goal {
    radius: f32,
}

#[derive(Component)]
struct Car {
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
    wants: Merch,
}

#[derive(Component)]
struct Projectile {
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
        (10, 100.0),
        (10, 200.0),
        (10, 300.0),
        (10, 300.0),
        (10, 200.0),
        (10, 100.0),
        (10, 0.0),
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

fn lv1_customers() -> Vec<(GameObject, Customer)> {
    vec![
        (
            GameObject {
                pos: Vec2::new(400., 500.),
                draw_scale: 0.15,
            },
            Customer { wants: Merch::Banana },
        ),
        (
            GameObject {
                pos: Vec2::new(-400., 100000.),
                draw_scale: 0.15,
            },
            Customer { wants: Merch::Banana },
        ),
        (
            GameObject {
                pos: Vec2::new(700., 200000.),
                draw_scale: 0.15,
            },
            Customer { wants: Merch::Banana },
        ),
    ]
}

fn turn_right(
    startpos: f32,
    width: f32,
    sharpness: f32,
    duration: usize,
) -> Vec<(usize, f32, f32)> {
    let mut res = vec![];
    let mut xpos = startpos;
    for _i in 0..duration {
        res.push((1, xpos, width));
        xpos += sharpness;
    }
    res
}

fn lv1_turns() -> Vec<(usize, f32, f32)> {
    // (how many blocks to render, x position of those blocks)
    let mut res = vec![
        (10, 0.0, 0.0),
        (10, 0.0, 10.0),
        (10, 0.0, 20.0),
        (10, 0.0, 0.0),
    ];
    // right
    res.extend(turn_right(0.0, 0.0, 20.0, 20));
    // strait
    res.extend(turn_right(20.0 * 20.0, 0.0, 0.0, 20));
    // left
    res.extend(turn_right(20.0 * 20.0, 0.0, -20.0, 40));
    //back right
    res.extend(turn_right(-20.0 * 20.0, 0.0, 20.0, 20));

    // big area
    res.extend(turn_right(0.0, 300.0, 0.0, 50));

    // right
    res.extend(turn_right(0.0, 0.0, 20.0, 20));
    // mismatched left
    res.extend(turn_right(0.0, 0.0, -20.0, 20));
    res.extend(vec![
        (10, 0.0, 0.0),
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
    ]);
    res
}

fn get_texture(all_sprites: &AllSprite, key: &str) -> Handle<Image> {
    all_sprites.map.get(key).unwrap().clone()
}

fn set_transformation(transform: &mut Transform, x: f32, z: f32, scale: f32, vel: f32) {
    let theta: f32 = (vel.max(0.0) / 10.0).atan() / 3.0;
    let denom: f32 = z * theta.sin() + 400.0 * theta.cos();
    transform.translation = Vec3::new(
        x / denom * 400.0,
        (z * theta.cos() - 400.0 * theta.sin()) / denom * (200.0 / 1.428),
        400.0 / denom,
    );
    if denom > 0.0 {
        transform.scale = 400.0 / denom * Vec3::new(scale, scale * theta.cos(), scale);
    } else {
        transform.scale = Vec3::ZERO;
    }
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
        let mut transform = Transform::from_xyz(xpos, HEIGHT_OF_WALL, -1.);
        transform.scale = Vec3::new(0.1, 0.1, 0.1);
        for _n in 0..num {
            commands.spawn((
                PartOfLevel,
                GameObject {
                    pos: Vec2::new(xpos + left_side - more_offset, ypos),
                    draw_scale: 0.1,
                },
                SpriteBundle {
                    texture: asset_server.load("static-wall.png"),
                    transform,
                    ..default()
                },
                Obstacle,
            ));
            commands.spawn((
                PartOfLevel,
                GameObject {
                    pos: Vec2::new(xpos - left_side + more_offset, ypos),
                    draw_scale: 0.1,
                },
                SpriteBundle {
                    texture: asset_server.load("static-wall.png"),
                    transform,
                    ..default()
                },
                Obstacle,
            ));

            ypos += HEIGHT_OF_WALL;
        }
    }
}

fn setup_fps_counter(commands: &mut Commands) {
    // create our UI root node
    // this is the wrapper/container for the text
    // create our text
    commands
        .spawn((
            FpsText,
            TextBundle {
                // use two sections, so it is easy to update just the number
                text: Text::from_sections([
                    TextSection {
                        value: "FPS: ".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            // if you want to use your game's font asset,
                            // uncomment this and provide the handle:
                            // font: my_font_handle
                            ..default()
                        },
                    },
                    TextSection {
                        value: " N/A".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            // if you want to use your game's font asset,
                            // uncomment this and provide the handle:
                            // font: my_font_handle
                            ..default()
                        },
                    },
                ]),
                ..Default::default()
            }.with_text_justify(JustifyText::Left)
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            }),
        ));
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
        "finish.png",
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
    setup_fps_counter(&mut commands);
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

    // make text that says "use J and K to shoot bananas"
    let mut transform = Transform::from_xyz(0., 0., 3.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            "Use J and K to shoot bananas",
            TextStyle {
                font_size: 50.0,
                color: Color::GOLD,
                ..Default::default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            left: Val::Percent(20.0),
            ..default()
        }),
        PartOfStart,
    ));

    // make text that says "everything MUST GO!"
    let mut transform = Transform::from_xyz(0., 0., 3.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            "Everything MUST GO!",
            TextStyle {
                font_size: 50.0,
                color: Color::GOLD,
                ..Default::default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(20.0),
            left: Val::Percent(20.0),
            ..default()
        }),
        PartOfStart,
    ));
}

fn setup_customers(commands: &mut Commands, all_sprites: &AllSprite) {
    for (entity, customer) in lv1_customers() {
        let mut transform = Transform::from_xyz(entity.pos.x, entity.pos.y, 1.0);
        transform.scale = Vec3::new(1.0, 1.0, 1.0) * 0.15;
        commands.spawn((
            PartOfLevel,
            entity.clone(),
            SpriteBundle {
                texture: get_texture(all_sprites, "banana-car.png"),
                transform,
                ..default()
            },
            customer.clone(),
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
        PartOfLevel,
        GameObject {
            pos: Vec2::new(100., 0.),
            draw_scale: 0.2,
        },
        SpriteBundle {
            texture: get_texture(all_sprites, "racecar_center.png"),
            transform,
            ..default()
        },
        Car {
            vel: Vec2::new(0., 0.),
            direction: Vec2::new(0., 1.),
            base_acc: 1.,
            top_speed: 40.,
            steer_strength: 0.0012,
            drift_strength: 0.06,
            projectile_speed: 100.0,
            ammo: lv1_ammo(),
            start_time: 0.0,
        },
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
    transform.scale = Vec3::new(1.0, 1.0, 1.0) * 0.2;
    // green circle for goal
    commands.spawn((
        PartOfLevel,
        GameObject {
            pos: Vec2::new(100., 100000.),
            draw_scale: 0.2,
        },
        SpriteBundle {
            texture: get_texture(all_sprites, "finish.png"),
            ..default()
        },
        Goal {
            radius: 400.,
        },
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
    mut sprite_position: Query<(&mut GameObject, &mut Car, &mut Handle<Image>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    sprites: Query<&AllSprite>,
) {
    for (mut car_object, mut car, mut texture) in &mut sprite_position {
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

        let mut car_velocity_update = Vec2::new(0.0, 0.0);
        if keyboard_input.pressed(KeyCode::KeyW) {
            let mut min2 = car.vel.length() / 10.0;
            if min2 > 1.0 {
                min2 = 1.0;
            }
            if min2 < 0.1 {
                min2 = 0.1;
            }

            car_velocity_update += car.direction * car.base_acc * min2;
        }
        if car.vel.length() > 0.000001 {
            car_velocity_update -=
                car.vel.angle_between(car.direction).abs() * car.vel * car.drift_strength;
        }

        car.vel += car_velocity_update;

        // Limit the length of the vector to car.top_speed
        if car.vel.length() > car.top_speed {
            car.vel = car.vel.normalize() * car.top_speed;
        }

        car_object.pos = car_object.pos + car.vel;

        /*
        TODO add accel changes.
        vel += accel * time.delta_seconds(); // Check if this works in direction we need
        pos += vel * time.delta_seconds();
        */
    }
}

fn draw_objects(
    mut entity: Query<(&GameObject, &mut Transform), Without<Car>>,
    mut car_entity: Query<(&GameObject, &mut Transform, &Car)>,
) {
    if let Ok((car_object, mut car_transform, car)) = car_entity.get_single_mut() {
        // Draw other objects
        for (object, mut transform) in &mut entity {
            set_transformation(&mut transform, object.pos.x, object.pos.y - car_object.pos.y, object.draw_scale, car.vel.y);
        }
        // Draw car
        set_transformation(&mut car_transform, car_object.pos.x, 0.0, car_object.draw_scale, car.vel.y);
        car_transform.rotation =
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

fn collision_update_system(
    obstacles: Query<(&GameObject, &Obstacle)>,
    car_entity: Query<(&GameObject, &Car)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut comands: Commands,
    time: Res<Time>,
) {
    if let Ok((car_object, car)) = car_entity.get_single() {
        let mut game_over = false;
        for (obstacle_object, _) in &obstacles {
            if car_object.pos.distance(obstacle_object.pos) < 75. {
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
        if let Ok(audio_sink) = audio.get_single() {
            audio_sink.pause();
        }
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

fn projectile_update(mut projectiles: Query<(&mut GameObject, &Projectile)>) {
    for (mut projectile_object, projectile) in &mut projectiles {
        projectile_object.pos = projectile_object.pos + projectile.vel;
    }
}

fn detect_shoot_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    all_sprites: Query<&AllSprite>,
    mut car_entity: Query<(&GameObject, &mut Car)>,
) {
    let (car_object, mut car) = car_entity.single_mut();
    for keycode in [KeyCode::KeyK, KeyCode::KeyJ] {
        if car.ammo.get(&Merch::Banana).unwrap_or(&0) != &0 && keyboard_input.just_pressed(keycode)
        {
            let mut transform = Transform::from_xyz(car_object.pos.x, car_object.pos.y, 1.0);
            transform.scale = Vec3::new(0.04, 0.04, 0.04);
            let angle = if keycode == KeyCode::KeyK {
                -std::f32::consts::FRAC_PI_2
            } else {
                std::f32::consts::FRAC_PI_2
            };
            commands.spawn((
                PartOfLevel,
                GameObject {
                    pos: car_object.pos,
                    draw_scale: 0.05,
                },
                SpriteBundle {
                    texture: get_texture(all_sprites.get_single().unwrap(), "banana.png"),
                    transform,
                    ..default()
                },
                Projectile {
                    // rotate direction so it shoots from right if J is pressed
                    vel: car.direction.rotate(Vec2::from_angle(angle)) * car.projectile_speed,
                    merch: Merch::Banana,
                },
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
    projectiles: Query<(Entity, &GameObject, &Projectile)>,
    customers: Query<(Entity, &GameObject, &Customer)>,
    obstacles: Query<(Entity, &GameObject, &Obstacle)>,
) {
    for (projectile_entity, projectile_object, projectile) in &mut projectiles.iter() {
        for (customer_entity, customer_object, customer) in &mut customers.iter() {
            if projectile_object.pos.distance(customer_object.pos) < 100. && projectile.merch == customer.wants {
                commands.entity(projectile_entity).despawn();
                commands.entity(customer_entity).despawn();
            }
        }
    }

    for (projectile_entity, projectile_object, projectile) in &mut projectiles.iter() {
        for (_obstacle_entity, obstacle_object, obstacle) in &mut obstacles.iter() {
            if projectile_object.pos.distance(obstacle_object.pos) < 100. {
                commands.entity(projectile_entity).despawn();
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
    car_entity: Query<(&GameObject, &Car)>,
    goals: Query<(&GameObject, &Goal)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    if let Ok((car_object, car)) = car_entity.get_single() {
        for (goal_object, goal) in &goals {
            if car_object.pos.y > goal_object.pos.y && car_object.pos.distance(goal_object.pos) < goal.radius {
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
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        // try to get a "smoothed" FPS value from Bevy
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            // Format the number as to leave space for 4 digits, just in case,
            // right-aligned and rounded. This helps readability when the
            // number changes rapidly.
            text.sections[1].value = format!("{value:>4.0}");

            // Let's make it extra fancy by changing the color of the
            // text according to the FPS value:
            text.sections[1].style.color = if value >= 120.0 {
                // Above 120 FPS, use green color
                Color::rgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                // Between 60-120 FPS, gradually transition from yellow to green
                Color::rgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
            } else if value >= 30.0 {
                // Between 30-60 FPS, gradually transition from red to yellow
                Color::rgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
            } else {
                // Below 30 FPS, use red color
                Color::rgb(1.0, 0.0, 0.0)
            }
        } else {
            text.sections[1].value = " Failed".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}
