//! Renders a 2D scene containing a single, moving sprite.

use bevy::{
    asset::AssetMetaCheck, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::*, utils::hashbrown::HashMap
};

const HEIGHT_OF_WALL: f32 = 160.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, States)]
enum AppState {
    EndLevel {
        level: usize,
        did_win: bool,
        did_finish: bool,
        score: usize,
    },
    StartLevel(usize),
    Game,
}

fn main() {
    let draw_level = (
        car_draw,
        obstacle_draw,
        hazard_draw,
        customer_draw,
        projectile_draw,
        draw_num_ammo,
        goal_draw,
        fps_text_update_system,
    );
    App::new()
        // Wasm builds will check for meta files (that don't exist) if this isn't set.
        // This causes errors and even panics on web build on itch.
        // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
        .insert_resource(AssetMetaCheck::Never)
        .insert_state(AppState::StartLevel(0))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, initial_setup)
        .add_systems(
            Update,
            (
                sprite_movement,
                text_update_system,
                collision_update_system,
                collision_update_system_hazards, // Cheers Dhruba :)
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

#[derive(Component)]
struct SaveData {
    pub scores: Vec<usize>,
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
struct Obstacle {
    pos: Vec2,
    bounce_dir: f32,
}

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct Goal {
    pos: Vec2,
    radius: f32,
}

#[derive(Component)]
struct Hazard {
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
    projectile_speed: f32,
    ammo: HashMap<Merch, usize>,
    frames_elapsed: usize,
    hard_mode: bool,
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

fn lv1_ammo() -> HashMap<Merch, usize> {
    vec![(Merch::Banana, 10)].into_iter().collect()
}

enum Placement {
    Customer { xpos: f32 },
    Goal { xpos: f32 },
    HangryCone { xpos: f32 },
}

fn lv1_turns() -> Vec<(usize, f32, f32, Vec<Placement>)> {
    // (how many blocks to render, x position of those blocks, size of gap, customers (x, y))
    let base_width = 400.0;
    let mut res = vec![
        (10, 0.0, base_width, vec![]),
        (10, 0.0, base_width, vec![]),
        (10, 0.0, base_width, vec![]),
        (10, 0.0, base_width, vec![]),
    ];
    let sharpness_easy = 30.0;
    let base_width = 400.0;
    // target
    res.push((3, 0.0, base_width + 300.0, vec![]));
    res.push((
        2,
        0.0,
        base_width + 300.0,
        vec![Placement::Customer { xpos: -500.0 }],
    ));
    // right
    res.push((20, sharpness_easy, base_width, vec![]));
    // strait
    res.push((
        20,
        0.0,
        base_width,
        vec![Placement::HangryCone { xpos: (0.0) }],
    ));
    // target
    res.push((3, 0.0, base_width + 300.0, vec![]));
    res.push((
        2,
        0.0 * sharpness_easy,
        base_width + 300.0,
        vec![Placement::Customer { xpos: 500.0 }],
    ));
    // left
    res.push((40, -sharpness_easy, base_width, vec![]));
    //back right
    res.push((20, sharpness_easy, base_width, vec![]));

    // big area
    res.push((5, 0.0, 1000.0, vec![]));
    res.push((0, 0.0, 0.0, vec![Placement::Customer { xpos: -800.0 }]));
    res.push((0, 0.0, 0.0, vec![Placement::Customer { xpos: 800.0 }]));
    res.push((10, 0.0, 1000.0, vec![]));
    res.push((0, 0.0, 0.0, vec![Placement::Customer { xpos: -800.0 }]));
    res.push((0, 0.0, 0.0, vec![Placement::Customer { xpos: 800.0 }]));
    res.push((5, 0.0, 1000.0, vec![]));

    res.push((50, 0.0, 700.0, vec![]));

    // make next one flush with right wall, leaving gap on left
    res.push((1, (700.0 - 500.0) * 2.0, 15000.0, vec![]));
    // right
    res.push((5, sharpness_easy, 500.0, vec![]));
    // target is outside of the lane
    res.push((0, 0.0, 0.0, vec![Placement::Customer { xpos: -1500.0 }]));
    res.push((15, sharpness_easy, base_width, vec![]));

    // strait section
    res.push((20, 0.0, 600.0, vec![]));

    // make flush with wall but leave gap on right
    res.push((1, -(600.0 - 400.0) * 2.0, 15000.0, vec![]));
    // left
    res.push((5, -sharpness_easy, 400.0, vec![]));
    // target is outside of the lane
    res.push((0, 0.0, 0.0, vec![Placement::Customer { xpos: 1500.0 }]));
    res.push((15, -sharpness_easy, base_width, vec![]));

    let sharper = 50.0;
    // hard zig zags
    res.push((15, sharper, base_width, vec![]));
    res.push((15, -sharper, base_width, vec![]));
    res.push((15, sharper, base_width, vec![]));
    res.push((15, -sharper, base_width, vec![]));
    res.push((15, sharper, base_width, vec![]));

    // strait at the end
    res.push((10, 0.0, base_width, vec![]));

    // two targets
    res.push((3, 0.0, base_width + 300.0, vec![]));
    res.push((
        1,
        0.0,
        base_width + 300.0,
        vec![
            Placement::Customer { xpos: -500.0 },
            Placement::Customer { xpos: 500.0 },
        ],
    ));
    res.push((3, 0.0, base_width + 300.0, vec![]));

    // last strait before goal
    res.push((10, 0.0, base_width, vec![]));
    let boxsize = 20;
    // goal inside a box
    for i in 0..boxsize {
        res.push((1, 0.0, base_width + ((i as f32) * 20.0), vec![]));
    }
    let boxw = base_width + ((boxsize as f32) * 20.0);

    // goal box middle
    res.push((10, 0.0, boxw, vec![]));
    res.push((10, 0.0, boxw, vec![Placement::Goal { xpos: 0.0 }]));

    // end of the box
    for i in 0..100 {
        res.push((1, 0.0, boxw - ((i as f32) * 20.0), vec![]));
    }

    res
}

fn get_texture(all_sprites: &AllSprite, key: &str) -> Handle<Image> {
    all_sprites.map.get(key).unwrap().clone()
}

fn set_transformation(
    transform: &mut Transform,
    pos: &Vec2,
    scale: f32,
    car: &Car,
    sprite_size: Vec2,
) {
    let theta: f32 =
        (car.vel.y.max(0.) / 10.).atan() / 2. + (car.pos.y.max(0.) / 10000.).atan() / 6.;
    let denom: f32 = (pos.y - car.pos.y) * theta.sin() + 400. * theta.cos();
    let car_xpos: f32 = 250. * (car.pos.x / 250.).atan();
    transform.translation = Vec3::new(
        (pos.x - car.pos.x + car_xpos) / denom * 400.,
        ((pos.y - car.pos.y) * theta.cos() - 400. * theta.sin()) / denom * (200. / 1.428),
        400. / denom,
    );
    if denom > 0. {
        transform.scale = 400. / denom * Vec3::new(scale, scale * theta.cos(), scale);
    } else {
        transform.scale = Vec3::ZERO;
    }
}

fn setup_obstacles(commands: &mut Commands, all_sprites: &AllSprite) {
    let mut transform = Transform::from_xyz(0., 20., -1.0);
    transform.scale = Vec3::new(0.1, 0.1, 0.1);

    let mut ypos = -100.0;
    let mut current_xpos = 0.0;

    for (num, xpos, more_offset, customers) in lv1_turns() {
        for placement in customers {
            match placement {
                Placement::Customer { xpos: customerx } => {
                    let customer = Customer {
                        pos: Vec2::new(current_xpos + customerx, ypos),
                        wants: Merch::Banana,
                    };
                    let mut transform = Transform::from_xyz(customer.pos.x, customer.pos.y, 1.0);
                    transform.scale = Vec3::new(1.0, 1.0, 1.0) * 0.15;
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
                    let mut transform =
                        Transform::from_xyz(customer.pos.x, customer.pos.y + 100., 3.0);
                    transform.scale = Vec3::new(1.0, 1.0, 1.0) * 0.15;
                    let bubble_pos = Vec2::new(customer.pos.x, customer.pos.y + 100.);
                    commands.spawn((
                        SpriteBundle {
                            texture: get_texture(all_sprites, "banana-speech.png"),
                            transform,
                            ..default()
                        },
                        CustomerBubble { pos: bubble_pos },
                        PartOfLevel,
                    ));
                }
                Placement::Goal { xpos: goal_xpos } => {
                    let mut transform = Transform::from_xyz(100., 10000., 2.0);
                    transform.scale = Vec3::new(1.0, 1.0, 1.0) * 0.2;
                    commands.spawn((
                        SpriteBundle {
                            texture: get_texture(all_sprites, "finish.png"),
                            ..default()
                        },
                        Goal {
                            pos: Vec2::new(current_xpos + goal_xpos, ypos),
                            radius: 300.,
                        },
                        PartOfLevel,
                    ));
                }
                Placement::HangryCone { xpos: cone_xpos } => {
                    let mut transform = Transform::from_xyz(100., 10000., 2.0);
                    transform.scale = Vec3::new(1.0, 1.0, 1.0) * 0.2;
                    commands.spawn((
                        SpriteBundle {
                            texture: get_texture(all_sprites, "Angry-bougie-cone.png"),
                            ..default()
                        },
                        Hazard {
                            pos: Vec2::new(current_xpos + cone_xpos, ypos),
                        },
                        PartOfLevel,
                    ));
                }
            }
        }

        let mut transform = Transform::from_xyz(xpos, HEIGHT_OF_WALL, -1.);
        transform.scale = Vec3::new(0.1, 0.1, 0.1);
        for _n in 0..num {
            commands.spawn((
                SpriteBundle {
                    texture: get_texture(all_sprites, "static-wall.png"),
                    transform,
                    ..default()
                },
                Obstacle {
                    pos: Vec2::new(current_xpos + xpos - more_offset, ypos),
                    bounce_dir: more_offset.signum(),
                },
                PartOfLevel,
            ));
            commands.spawn((
                SpriteBundle {
                    texture: get_texture(all_sprites, "static-wall.png"),
                    transform,
                    ..default()
                },
                Obstacle {
                    pos: Vec2::new(current_xpos + xpos + more_offset, ypos),
                    bounce_dir: -more_offset.signum(),
                },
                PartOfLevel,
            ));

            current_xpos += xpos;
            ypos += HEIGHT_OF_WALL;
        }
    }
}

fn setup_fps_counter(commands: &mut Commands) {
    // create our UI root node
    // this is the wrapper/container for the text
    // create our text
    commands.spawn((
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
        }
        .with_text_justify(JustifyText::Left)
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
        "static-wall.png",
        "Angry-bougie-cone.png",
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
    setup_level(&mut commands, &all_sprites);
    setup_save(&mut commands);
    commands.spawn(all_sprites);
}

fn setup_save(commands: &mut Commands) {
    commands.spawn((SaveData { scores: vec![] },));
}

fn setup_endlevel(
    commands: &mut Commands,
    did_win: bool,
    did_finish: bool,
    mut save: Query<&mut SaveData>,
    frames_elapsed: usize,
) {
    let text = if did_win {
        let mut save = save.single_mut();
        save.scores.push(0);
        "You won!"
    } else if did_finish {
        "You lost! You didn't deliver to all 10 customers!"
    } else {
        "You lost! You crashed!"
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

    // add a text component "Press space to restart"
    let mut transform = Transform::from_xyz(0., 0., 3.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            "Press Space to Restart",
            TextStyle {
                font_size: 50.0,
                color: Color::GOLD,
                ..Default::default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(60.0),
            left: Val::Percent(20.0),
            ..default()
        }),
        PartOfEndLevel,
    ));

    // Add text component that shows best 5 scores
    let mut transform = Transform::from_xyz(0., 0., 3.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    let mut text = "Best Scores:\n".to_string();
    let mut save = save.single_mut();
    save.scores.sort();
    save.scores.reverse();
    for score in save.scores.iter().take(5) {
        text.push_str(&format!("{}\n", 60.0 * (*score as f64)));
    }
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
            top: Val::Percent(30.0),
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

    // make text "press h for hardcore mode"
    let mut transform = Transform::from_xyz(0., 0., 3.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            "Press H for Hardcore Mode",
            TextStyle {
                font_size: 50.0,
                color: Color::GOLD,
                ..Default::default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(60.0),
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
            "Controls: W, A, D, J, K",
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
            "bananas NOW!",
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
            base_acc: 0.7,
            top_speed: 80.,
            steer_strength: 0.0012,
            drift_strength: 0.06,
            projectile_speed: 100.0,
            ammo: lv1_ammo(),
            frames_elapsed: 0,
            hard_mode: false,
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

fn setup_level(commands: &mut Commands, all_sprites: &AllSprite) {
    setup_car(commands, all_sprites);
    setup_obstacles(commands, all_sprites);

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

        car.pos = car.pos + car.vel;

        /*
        TODO add accel changes.
        vel += accel * time.delta_seconds(); // Check if this works in direction we need
        pos += vel * time.delta_seconds();
        */
    }
}

// Draw functions
fn car_draw(
    mut car_query: Query<(&Car, &mut Transform)>,
    assets: Res<Assets<Image>>,
    the_allsprite: Query<&AllSprite>,
) {
    if let Some(sprite) = assets.get(get_texture(
        the_allsprite.get_single().unwrap(),
        "racecar_center.png",
    )) {
        for (car, mut transform) in &mut car_query {
            // Update sprite
            set_transformation(&mut transform, &car.pos, 0.2, &car, sprite.size_f32());
            transform.rotation =
                Quat::from_rotation_z(car.direction.to_angle() - std::f32::consts::FRAC_PI_2);
        }
    }
}
fn obstacle_draw(
    mut obstacle_query: Query<(&Obstacle, &mut Transform)>,
    car: Query<&Car>,
    assets: Res<Assets<Image>>,
    the_allsprite: Query<&AllSprite>,
) {
    if let Some(sprite) = assets.get(get_texture(
        the_allsprite.get_single().unwrap(),
        "static-wall.png",
    )) {
        let car = car.get_single().unwrap();
        for (obstacle, mut transform) in &mut obstacle_query {
            set_transformation(&mut transform, &obstacle.pos, 0.1, car, sprite.size_f32());
        }
    }
}
fn hazard_draw(
    mut hazard_query: Query<(&Hazard, &mut Transform)>,
    car: Query<&Car>,
    assets: Res<Assets<Image>>,
    the_allsprite: Query<&AllSprite>,
) {
    if let Some(sprite) = assets.get(get_texture(
        the_allsprite.get_single().unwrap(),
        "Angry-bougie-cone.png",
    )) {
        let car = car.get_single().unwrap();
        for (obstacle, mut transform) in &mut hazard_query {
            set_transformation(&mut transform, &obstacle.pos, 0.1, car, sprite.size_f32());
        }
    }
}
fn customer_draw(
    mut customer_query: Query<(&Customer, &mut Transform)>,
    car: Query<&Car>,
    assets: Res<Assets<Image>>,
    the_allsprite: Query<&AllSprite>,
) {
    if let Some(sprite) = assets.get(get_texture(
        the_allsprite.get_single().unwrap(),
        "banana-car.png",
    )) {
        let car = car.get_single().unwrap();
        for (customer, mut transform) in &mut customer_query {
            set_transformation(&mut transform, &customer.pos, 0.1, car, sprite.size_f32());
        }
    }
}
fn projectile_draw(
    mut projectile_query: Query<(&Projectile, &mut Transform)>,
    car: Query<&Car>,
    assets: Res<Assets<Image>>,
    the_allsprite: Query<&AllSprite>,
) {
    if let Some(sprite) = assets.get(get_texture(
        the_allsprite.get_single().unwrap(),
        "banana.png",
    )) {
        let car = car.get_single().unwrap();
        for (projectile, mut transform) in &mut projectile_query {
            set_transformation(
                &mut transform,
                &projectile.pos,
                0.05,
                car,
                sprite.size_f32(),
            );
        }
    }
}
fn goal_draw(
    mut goal_query: Query<(&Goal, &mut Transform)>,
    car: Query<&Car>,
    assets: Res<Assets<Image>>,
    the_allsprite: Query<&AllSprite>,
) {
    if let Some(sprite) = assets.get(get_texture(
        the_allsprite.get_single().unwrap(),
        "finish.png",
    )) {
        let car = car.get_single().unwrap();
        for (goal, mut transform) in &mut goal_query {
            set_transformation(&mut transform, &goal.pos, 1.0, car, sprite.size_f32());
        }
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
            ((time.elapsed_seconds() - (car.single().frames_elapsed as f32) * (1.0 / 60.0))
                * 100.0)
                .floor()
                / 100.0
        );
    }
}

fn collision_update_system(
    obstacles: Query<&Obstacle>,
    mut car: Query<&mut Car>,
    mut next_state: ResMut<NextState<AppState>>,
    mut comands: Commands,
    time: Res<Time>,
    audio: Query<&AudioSink>,
    mut save: Query<&mut SaveData>,
) {
    let mut car = car.get_single_mut().unwrap();

    let mut game_over = false;
    for obstacle in &obstacles {
        if (obstacle.pos.x - car.pos.x).abs() < 100.
            && (obstacle.pos.y - car.pos.y).abs() < 2. * HEIGHT_OF_WALL
            && obstacle.bounce_dir * car.vel.x < 0.
        {
            car.vel.x = -0.9 * car.vel.x + 0.15 * obstacle.bounce_dir * car.top_speed;
            car.pos = car.pos + car.vel;
            car.vel.x *= 0.6;
            car.vel.y *= 0.3;

            if car.hard_mode {
                game_over = true;
            }
        }
    }

    if game_over {
        if let Ok(sink) = audio.get_single() {
            sink.pause();
        }

        next_state.set(AppState::EndLevel {
            level: 0,
            did_win: false,
            score: car.frames_elapsed as usize,
            did_finish: false,
        });

        setup_endlevel(&mut comands, false, false, save, car.frames_elapsed);
    }
}

fn collision_update_system_hazards(
    hazards: Query<&Hazard>,
    car: Query<&Car>,
    mut next_state: ResMut<NextState<AppState>>,
    mut comands: Commands,
    time: Res<Time>,
    audio: Query<&AudioSink>,
    mut save: Query<&mut SaveData>,
) {
    let car = car.get_single().unwrap();
    let mut game_over = false;
    for hazard in &hazards {
        if car.pos.distance(hazard.pos) < 75. * 3. {
            // Makes cone radius larger
            // Game over
            // TODO bounce, but game over in hardcore mode
            game_over = true;
        }
    }

    if game_over {
        if let Ok(sink) = audio.get_single() {
            sink.pause();
        }

        next_state.set(AppState::EndLevel {
            level: 0,
            did_win: false,
            score: car.frames_elapsed as usize,
            did_finish: false,
        });

        setup_endlevel(&mut comands, false, false, save, car.frames_elapsed);
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
    sprites: Query<&AllSprite>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in to_delete.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(AppState::StartLevel(0));
        // set car start time
        let mut car = car.single_mut();
        car.frames_elapsed = 0;

        // delete things part of the level
        for entity in to_delete.iter().chain(to_delete2.iter()) {
            commands.entity(entity).despawn();
        }
        setup_start(&mut commands, sprites.get_single().unwrap());
        setup_level(&mut commands, sprites.get_single().unwrap());
    }
}

fn check_start_level(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    to_delete: Query<Entity, With<PartOfStart>>,
    mut commands: Commands,
    audio: Query<&AudioSink>,
    mut car: Query<&mut Car>,
) {
    let h_pressed = keyboard_input.just_pressed(KeyCode::KeyH);
    if keyboard_input.just_pressed(KeyCode::Space) | h_pressed {
        audio.get_single().unwrap().play();
        for entity in to_delete.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(AppState::Game);
        // set car start time
        let mut car = car.single_mut();
        car.frames_elapsed = 0;
        car.hard_mode = keyboard_input.just_pressed(KeyCode::KeyH);
    }
}

fn projectile_update(mut projectiles: Query<&mut Projectile>) {
    for mut projectile in &mut projectiles {
        projectile.pos = projectile.pos + projectile.vel;
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
    obstacles: Query<(Entity, &Obstacle)>,
) {
    for (projectile_entity, projectile) in &mut projectiles.iter() {
        for (customer_entity, customer) in &mut customers.iter() {
            if projectile.pos.distance(customer.pos) < 200. && projectile.merch == customer.wants {
                commands.entity(projectile_entity).despawn();
                commands.entity(customer_entity).despawn();
            }
        }
    }

    for (projectile_entity, projectile) in &mut projectiles.iter() {
        for (_obstacle_entity, obstacle) in &mut obstacles.iter() {
            if projectile.pos.distance(obstacle.pos) < 100. {
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
    car: Query<&mut Car>,
    goals: Query<&Goal>,
    mut commands: Commands,
    customers: Query<&Customer>,
    save: Query<&mut SaveData>,
) {
    let num_customers_left = customers.iter().count();
    let car = car.iter().next().unwrap();
    for goal in goals.iter() {
        if car.pos.y > goal.pos.y && (car.pos.x - goal.pos.x).abs() < goal.radius {
            let did_win = num_customers_left == 0;

            next_state.set(AppState::EndLevel {
                level: 0,
                did_win,
                did_finish: true,
                score: car.frames_elapsed,
            });

            setup_endlevel(&mut commands, did_win, true, save, car.frames_elapsed);
            break;
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
