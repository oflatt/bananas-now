//! Renders a 2D scene containing a single, moving sprite.

use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, sprite_movement)
        .run();
}

#[derive(Component)]
struct Obstacle {
    pos: Vec2,
}

#[derive(Component)]
struct Car {
    pos: Vec2,
    vel: Vec2,
    direction: Vec2,
}

fn setup_obstacles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut transform = Transform::from_xyz(0., 20., 0.);
    transform.scale = Vec3::new(0.2, 0.2, 0.2);
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("cone.png"),
            transform,
            ..default()
        },
        Obstacle {
            pos: Vec2::new(100., 0.),
        },
    ));
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
        },
    ));
    setup_obstacles(commands, asset_server);
}

fn cursor_position(q_windows: &Query<&Window, With<PrimaryWindow>>) -> Vec2 {
    // Games typically only have one window (the primary window)
    if let Some(position) = q_windows.single().cursor_position() {
        position
    } else {
        Vec2::ZERO
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
        if keyboard_input.pressed(KeyCode::KeyA) {
            car.pos.x -= 1.;
        }

        car.pos.y = cursor_position(&q_window).y;
        transform.translation.y = car.pos.y;
        transform.translation.x = car.pos.x;
    }
}
