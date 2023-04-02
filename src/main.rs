#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
// use bevy_flycam::{MovementSettings, PlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod backgammon;

fn main() {
    let game = backgammon::Game::new();

    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(game)
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        // .add_plugin(PlayerPlugin)
        // .insert_resource(MovementSettings {
        //     sensitivity: 0.00015, // default: 0.00012
        //     speed: 12.0,          // default: 12.0
        // })
        .add_startup_system(spawn_board)
        .add_startup_system(spawn_pieces)
        .run();
}

fn spawn_board(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(-1.0, 1.0, 0.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    },));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/board.glb#Scene0"),
        ..default()
    });
}

pub fn spawn_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game: Res<backgammon::Game>,
) {
    let cp_handle = asset_server.load("models/piece.glb#Mesh0/Primitive0");
    let mut transform = Transform::from_xyz(0.0, -0.01, 0.0)
        .with_scale(Vec3::splat(0.002))
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));

    for (position, piece) in game.board.points.iter().enumerate() {
        let mut color = Color::WHITE;
        if *piece < 0 {
            color = Color::BLACK;
        }

        let position = position + 1_usize;
        let num_pieces = piece.unsigned_abs() as usize;

        for row in 1..=num_pieces {
            let [x, y] = get_piece_coordinate(position, row);
            transform.translation = Vec3::new(y, 0.0, x);

            let bundle = PbrBundle {
                mesh: cp_handle.clone(),
                material: materials.add(color.into()),
                transform,
                ..Default::default()
            };
            commands.spawn(bundle);
        }
    }
}

fn get_piece_coordinate(position: usize, row: usize) -> [f32; 2] {
    const DELTA_Y: f32 = 0.07;

    let mut coordinates: [f32; 2] = [0.0, 0.0];

    let mut y_start;
    let mut x_start;
    let mut x_end;

    if (1..=12).contains(&position) {
        y_start = -0.4;
        x_start = 0.067;
        x_end = 0.49;

        let delta = (x_end - x_start) / 5.0;
        let offset = -1.0 * (position as f32) + 6.0;
        coordinates[0] = x_start + delta * offset;
        coordinates[1] = y_start + DELTA_Y * (row - 1) as f32;

        if position >= 7 {
            coordinates[0] -= 0.039;
        }
    }

    if (13..=24).contains(&position) {
        y_start = 0.33;
        x_start = -0.48;
        x_end = -0.06;

        let delta = (x_end - x_start) / 5.0;
        let offset = 1.0 * (position as f32) - 1.0;
        coordinates[0] = x_start + delta * offset - 0.718 - 0.3 + 0.017;
        coordinates[1] = y_start - DELTA_Y * (row - 1) as f32;

        if position >= 19 {
            coordinates[0] += 0.039;
        }
    }

    coordinates
}
