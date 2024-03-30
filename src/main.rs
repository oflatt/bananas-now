//! Renders a 2D scene containing a single, moving sprite.

use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, sprite_movement)
        .run();
}

struct Obstacle {
    pos: Vec2,
}

#[derive(Component)]
struct Car {
    pos: Vec2,
    vel: Vec2, // Velocity is calculated
    direction: Vec2,
    base_acceleration: f32,
    // mass: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("icon.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Car {
            pos: Vec2::new(100., 0.),
            vel: Vec2::new(0., 0.),
            direction: Vec2::new(0., 1.),
            base_acceleration: 1.,
        },
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

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(
    _time: Res<Time>,
    mut sprite_position: Query<(&mut Car, &mut Transform)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (mut car, mut transform) in &mut sprite_position { // Finds the car
        if keyboard_input.pressed(KeyCode::KeyA) {
            car.pos.x -= 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            car.pos.x += 1.;
        } // TODO make this rotation around direction

        // Physics processing
        car.pos.y = cursor_position(&q_window).y;
        transform.translation.y = car.pos.y;
        transform.translation.x = car.pos.x;

        /*
        TODO add accel changes.
        vel += accel * time.delta_seconds(); // Check if this works in direction we need
        pos += vel * time.delta_seconds();
        */
    }
}
