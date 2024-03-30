//! Renders a 2D scene containing a single, moving sprite.

use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (sprite_movement, text_update_system, obstacle_update_system),
        )
        .run();
}

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
    base_acceleration: f32,
    top_speed: f32,
    // mass: f32,
}

fn lv1_turns() -> Vec<(usize, f32)> {
    vec![(10, 0.0), (20, 50.0), (30, 100.0)]
}

fn setup_obstacles(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let mut transform = Transform::from_xyz(0., 20., 0.);
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
    let left_side = -400.0;
    // place level obstacles
    for (num, xpos) in lv1_turns() {
        let mut transform = Transform::from_xyz(xpos, 20., 0.);
        transform.scale = Vec3::new(0.1, 0.1, 0.1);
        for _n in 0..num {
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("cone.png"),
                    transform,
                    ..default()
                },
                Obstacle {
                    pos: Vec2::new(xpos + left_side, ypos),
                },
            ));
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("cone.png"),
                    transform,
                    ..default()
                },
                Obstacle {
                    pos: Vec2::new(xpos - left_side, ypos),
                },
            ));

            ypos += 100.0;
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut transform = Transform::from_xyz(100., 0., 0.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("car.png"),
            transform,
            ..default()
        },
        Car {
            pos: Vec2::new(100., 0.),
            vel: Vec2::new(0., 0.),
            direction: Vec2::new(0., 1.),
            base_acceleration: 1.,
            top_speed: 10.,
        },
    ));
    setup_obstacles(&mut commands, asset_server);

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
    ));
}

fn cursor_position(q_windows: &Query<&Window, With<PrimaryWindow>>) -> Vec2 {
    // Games typically only have one window (the primary window)
    if let Some(position) = q_windows.single().cursor_position() {
        position
    } else {
        Vec2::ZERO
    }
}

fn change_sprite(mut sprite: Query<&mut Handle<ColorMaterial>>, asset_server: Res<AssetServer>) {
    for (mut sprite) in &mut sprite {
        *sprite = asset_server.load("car.png");
    }
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(
    _time: Res<Time>,
    mut sprite_position: Query<(&mut Car, &mut Transform)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (mut car, mut transform) in &mut sprite_position {
        // Finds the car
        if keyboard_input.pressed(KeyCode::KeyA) {
            car.direction = Vec2::new(-1., 0.);
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            car.direction = Vec2::new(1., 0.);
        } // TODO make this rotation around direction

        // Physics processing
        car.pos.y = cursor_position(&q_window).y;

        let car_velocity_update = car.direction * car.base_acceleration;
        car.vel = car.vel + car_velocity_update;

        // Limit the length of the vector to car.top_speed
        if car.vel.length() > car.top_speed {
            car.vel = car.vel.normalize() * car.top_speed;
        }

        car.pos = car.pos + car.vel;

        // Update sprite
        transform.translation.y = car.pos.y;
        transform.translation.x = car.pos.x;

        // Rotate the sprite toward the direction vector
        transform.rotate(Quat::from_rotation_z(
            car.direction.angle_between(Vec2::new(0., 1.)),
        ));

        /*
        TODO add accel changes.
        vel += accel * time.delta_seconds(); // Check if this works in direction we need
        pos += vel * time.delta_seconds();
        */
    }
}

fn text_update_system(time: Res<Time>, mut query: Query<&mut Text, With<TimerText>>) {
    for mut text in &mut query {
        text.sections[0].value = format!("Time: {}", time.elapsed_seconds().floor());
    }
}

fn obstacle_update_system(mut obstacles: Query<(&Obstacle, &mut Transform)>) {
    for (obstacle, mut transform) in &mut obstacles {
        transform.translation.x = obstacle.pos.x;
        transform.translation.y = obstacle.pos.y;
    }
}
