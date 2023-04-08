#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod events;
mod game;
mod ui;

use crate::ui::setup_ui;
use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use bevy_dice::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy_kira_audio::AudioPlugin;
use bevy_mod_picking::*;
use bevy_rapier3d::prelude::*;

use events::*;
use ui::*;

#[derive(Clone, Debug, Resource)]
pub(crate) struct GameResources {
    white_material: Handle<StandardMaterial>,
    black_material: Handle<StandardMaterial>,
    highlighted_material: Handle<StandardMaterial>,
    candidate_material: Handle<StandardMaterial>,
    checkers_model: Handle<Mesh>,
}

impl FromWorld for GameResources {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        let checkers_model = asset_server.load("models/piece.glb#Mesh0/Primitive0");

        GameResources {
            white_material: materials.add(bevy::prelude::Color::WHITE.into()),
            black_material: materials.add(bevy::prelude::Color::BLACK.into()),
            highlighted_material: materials.add(bevy::prelude::Color::RED.into()),
            candidate_material: materials
                .add(bevy::prelude::Color::rgba(0.0, 0.9, 0.0, 0.5).into()),
            checkers_model,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub(crate) struct Piece {
    row: usize,
    position: usize,
    color: game::Color,
    highlighted: bool,
    candidate: bool,
    chosen: bool,
}

impl Piece {
    fn board_coordinates(&self) -> [f32; 2] {
        const DELTA_Y: f32 = 0.07;

        let mut coordinates: [f32; 2] = [0.0, 0.0];

        let mut y_start;
        let mut x_start;
        let mut x_end;

        if (1..=12).contains(&self.position) {
            y_start = -0.34;
            x_start = 0.08;
            x_end = 0.533;

            let delta = (x_end - x_start) / 5.0;
            let offset = -1.0 * (self.position as f32) + 6.0;
            coordinates[0] = x_start + delta * offset;
            coordinates[1] = y_start + DELTA_Y * (self.row - 1) as f32;

            if self.position >= 7 {
                coordinates[0] -= 0.06;
            }
        }

        if (13..=24).contains(&self.position) {
            y_start = 0.34;
            x_start = -0.533;
            x_end = -0.08;

            let delta = (x_end - x_start) / 5.0;
            let offset = 1.0 * (self.position as f32) - 1.0;
            coordinates[0] = x_start + delta * offset - 0.718 - 0.3 + 0.017 - 0.06;
            coordinates[1] = y_start - DELTA_Y * (self.row - 1) as f32;

            if self.position >= 19 {
                coordinates[0] += 0.039;
            } else {
                coordinates[0] -= 0.022;
            }
        }

        coordinates
    }
}

fn spawn_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut start_game_event_writer: EventWriter<StartGameEvent>,
) {
    commands
        .spawn((Camera3dBundle {
            transform: Transform::from_xyz(-1.7, 1.7, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },))
        .insert(PickingCameraBundle::default());

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
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/board.glb#Scene0"),
            transform: Transform::from_xyz(0.0, 0.04, 0.0)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(0.6)),
            ..default()
        })
        .insert(Name::new("Board"));

    // Spawn lights
    commands
        .spawn(SpotLightBundle {
            transform: Transform::from_xyz(0.0, 1.0, 3.0),
            ..Default::default()
        })
        .insert(Name::new("Spotlight"));

    start_game_event_writer.send(StartGameEvent);
}

pub(crate) fn spawn_piece(commands: &mut Commands, piece: Piece, game_resources: GameResources) {
    let [x, y] = piece.board_coordinates();

    let transform = Transform::from_xyz(y, 0.0, x)
        .with_scale(Vec3::splat(0.03))
        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI));

    let mut material = match piece.color {
        game::Color::White => game_resources.white_material.clone(),
        game::Color::Black => game_resources.black_material.clone(),
    };

    if piece.highlighted {
        material = game_resources.highlighted_material.clone();
    }

    if piece.candidate {
        material = game_resources.candidate_material.clone();
    }

    let bundle = PbrBundle {
        mesh: game_resources.checkers_model,
        material,
        transform,
        ..Default::default()
    };

    let mut cmd = commands.spawn(bundle);

    cmd.insert(Name::new("Piece")).insert(piece);

    if piece.highlighted || piece.candidate {
        cmd.insert(PickableBundle::default());
    }
}

pub(crate) fn spawn_pieces(
    mut commands: Commands,
    game: ResMut<game::Game>,
    game_resources: Res<GameResources>,
) {
    for (position, piece) in game.board.points.iter().enumerate() {
        let mut color = game::Color::White;
        if *piece < 0 {
            color = game::Color::Black;
        }

        let position = position + 1_usize;
        let num_pieces = piece.unsigned_abs() as usize;

        for row in 1..=num_pieces {
            spawn_piece(
                &mut commands,
                Piece {
                    position,
                    row,
                    color,
                    highlighted: false,
                    candidate: false,
                    chosen: false,
                },
                game_resources.clone(),
            );
            // break;
        }

        // break;
    }
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugin(DicePlugin)
        .insert_resource(DicePluginSettings {
            render_size: (640, 640),
            number_of_fields: 1,
            dice_scale: 0.15,
            start_position: Vec3::new(-1.0, 0.0, -0.3),
            ..default()
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(game::Game::new())
        .add_event::<HighlightPickablePiecesEvent>()
        .add_event::<DisplayPossibleMovesEvent>()
        .add_event::<MovePieceEvent>()
        .add_event::<MovePieceEndEvent>()
        .add_event::<TurnStartEvent>()
        .add_event::<GameOverEvent>()
        .add_event::<StartGameEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(AudioPlugin)
        .add_plugins(DefaultPickingPlugins)
        .init_resource::<GameResources>()
        .add_startup_system(spawn_board)
        .add_startup_system(spawn_pieces)
        .add_startup_system(setup_ui)
        .add_system(ui_logic)
        .add_system(event_dice_roll_result)
        .add_system(event_dice_rolls_complete)
        .add_system(handle_hightlight_choosable_pieces)
        .add_system(handle_piece_picking.in_base_set(CoreSet::PostUpdate))
        .add_system(handle_display_possible_moves)
        .add_system(handle_move_piece_event)
        .add_system(handle_move_piece_end_event)
        .add_system(handle_dice_roll_start_event)
        .add_system(handle_turn_start_event)
        .add_system(handle_game_over_event)
        .add_system(handle_start_game_event)
        .run();
}
