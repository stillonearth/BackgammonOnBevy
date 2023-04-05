#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use bevy_dice::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::*;
use bevy_rapier3d::prelude::*;

use game::GameLogEntry;

use std::time::Duration;

mod game;

fn main() {
    let game = game::Game::new();

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
        .insert_resource(game)
        .insert_resource(GameUIState::None)
        // .insert_resource(SelectedPiece { entity: None })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugins(DefaultPickingPlugins)
        .init_resource::<GameResources>()
        .add_startup_system(spawn_board)
        .add_startup_system(spawn_pieces)
        .add_startup_system(setup_ui)
        .add_system(ui_logic)
        .add_system(event_dice_roll_result)
        .add_system(event_dice_rolls_complete)
        .add_system(hightlight_choosable_pieces)
        .add_system(handle_piece_picking_event.in_base_set(CoreSet::PostUpdate))
        .run();
}

fn spawn_board(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((Camera3dBundle {
            transform: Transform::from_xyz(-1.5, 1.5, 0.0)
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
            transform: Transform::from_xyz(0.0, 0.03, 0.0),
            ..default()
        })
        .insert(Name::new("Board"));
}

#[derive(Clone, Debug, Resource)]
struct GameResources {
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
struct Piece {
    row: usize,
    position: usize,
    color: game::Color,
    highlighted: bool,
    candidate: bool,
}

impl Piece {
    fn board_coordinates(&self) -> [f32; 2] {
        const DELTA_Y: f32 = 0.07;

        let mut coordinates: [f32; 2] = [0.0, 0.0];

        let mut y_start;
        let mut x_start;
        let mut x_end;

        if (1..=12).contains(&self.position) {
            y_start = -0.4;
            x_start = 0.067;
            x_end = 0.49;

            let delta = (x_end - x_start) / 5.0;
            let offset = -1.0 * (self.position as f32) + 6.0;
            coordinates[0] = x_start + delta * offset;
            coordinates[1] = y_start + DELTA_Y * (self.row - 1) as f32;

            if self.position >= 7 {
                coordinates[0] -= 0.039;
            }
        }

        if (13..=24).contains(&self.position) {
            y_start = 0.33;
            x_start = -0.48;
            x_end = -0.06;

            let delta = (x_end - x_start) / 5.0;
            let offset = 1.0 * (self.position as f32) - 1.0;
            coordinates[0] = x_start + delta * offset - 0.718 - 0.3 + 0.017;
            coordinates[1] = y_start - DELTA_Y * (self.row - 1) as f32;

            if self.position >= 19 {
                coordinates[0] += 0.039;
            }
        }

        coordinates
    }
}

fn spawn_piece(commands: &mut Commands, piece: Piece, game_resources: GameResources) {
    let [x, y] = piece.board_coordinates();

    let mut transform = Transform::from_xyz(0.0, 0.0, 0.0)
        .with_scale(Vec3::splat(0.002))
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));

    transform.translation = Vec3::new(y, 0.03, x);

    let mut material = match piece.color {
        game::Color::WHITE => game_resources.white_material.clone(),
        game::Color::BLACK => game_resources.black_material.clone(),
    };

    if piece.highlighted {
        material = game_resources.highlighted_material.clone();
    }

    if piece.candidate {
        material = game_resources.candidate_material.clone();
    }

    let bundle = PbrBundle {
        mesh: game_resources.checkers_model.clone(),
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

fn spawn_pieces(mut commands: Commands, game: Res<game::Game>, game_resources: Res<GameResources>) {
    for (position, piece) in game.board.points.iter().enumerate() {
        let mut color = game::Color::WHITE;
        if *piece < 0 {
            color = game::Color::BLACK;
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
                },
                game_resources.clone(),
            );
        }
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct LabelPlayerTurn;

#[derive(Component)]
struct ButtonRollDice;

#[derive(Component)]
struct LabelMoveStack;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "BackgammonOnBevy",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .insert(Name::new("Title"));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Start,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle::from_section(
                    "Turn: White",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ))
                .insert(LabelPlayerTurn);
        })
        .insert(Name::new("TurnIndicator"));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle::from_section(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ))
                .insert(LabelMoveStack);
        })
        .insert(Name::new("Move Stack"));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::End,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Roll Dice",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                })
                .insert(ButtonRollDice);
        })
        .insert(Name::new("BottomBar"));
}

#[allow(clippy::type_complexity)]
fn ui_logic(
    mut commands: Commands,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut label_set: ParamSet<(
        Query<&mut Text, With<LabelPlayerTurn>>,
        Query<&mut Text, With<LabelMoveStack>>,
    )>,
    mut button_roll_dice_query: Query<&mut Visibility, With<ButtonRollDice>>,
    mut ev_dice_started: EventWriter<DiceRollStartEvent>,
    mut game: ResMut<game::Game>,
) {
    for (_entity, interaction, mut color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();

                let num_dice: Vec<usize> = vec![2, 2];

                ev_dice_started.send(DiceRollStartEvent { num_dice });
                game.dice_rolled = true;

                commands.spawn(()).insert(DiceRollTimer {
                    timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
                });
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }

    for mut text in &mut label_set.p0().iter_mut() {
        text.sections[0].value = format!("Turn: {:?}", game.player);
    }

    if game.dice_rolled && !game.dice_rolls.is_empty() {
        for mut text in &mut label_set.p1().iter_mut() {
            text.sections[0].value = format!("Move Stack: {:?}", game.dice_rolls);
        }
    }

    for mut visibility in &mut button_roll_dice_query.iter_mut() {
        if game.dice_rolled {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Inherited;
        }
    }
}

pub(crate) fn event_dice_roll_result(
    mut dice_rolls: EventReader<DiceRollResult>,
    mut game: ResMut<game::Game>,
) {
    let player = game.player;
    for event in dice_rolls.iter() {
        game.game_log.push(GameLogEntry {
            player,
            dice_rolls: event.values[0].clone(),
        });
    }
}

#[derive(Component)]
pub(crate) struct DiceRollTimer {
    timer: Timer,
}

pub(crate) fn event_dice_rolls_complete(
    mut commands: Commands,
    mut dice_roll_timer_query: Query<(Entity, &mut DiceRollTimer)>,
    time: Res<Time>,
    mut game: ResMut<game::Game>,
) {
    for (entity, mut fuse_timer) in dice_roll_timer_query.iter_mut() {
        fuse_timer.timer.tick(time.delta());

        if fuse_timer.timer.finished() {
            let last_log_entry = game.game_log.last_mut().unwrap();
            let mut dice_rolls = last_log_entry.dice_rolls.clone();

            if dice_rolls[0] == dice_rolls[1] {
                dice_rolls.push(dice_rolls[0]);
                dice_rolls.push(dice_rolls[0]);
            }

            game.dice_rolls = dice_rolls;
            commands.entity(entity).despawn();
            commands.insert_resource(GameUIState::SelectPieceToMove);
        }
    }
}

#[allow(dead_code)]
#[derive(Resource, Debug, PartialEq, Eq)]
enum GameUIState {
    None,
    SelectPieceToMove,
    SelectPiecePosition,
}

fn hightlight_choosable_pieces(
    mut commands: Commands,
    game: Res<game::Game>,
    game_ui_state: Res<GameUIState>,
    mut query: Query<(Entity, &Piece)>,
    game_resources: Res<GameResources>,
) {
    if *game_ui_state.into_inner() != GameUIState::SelectPieceToMove {
        return;
    }

    let (choosable_points, _) = game.get_choosable_pieces();

    for (entity, piece) in &mut query.iter_mut() {
        for choosable_point in choosable_points.iter() {
            if piece.position == choosable_point[0] && piece.row == choosable_point[1] {
                if piece.highlighted {
                    continue;
                }
                commands.entity(entity).despawn();
                let mut new_piece = (*piece).clone();
                new_piece.highlighted = true;
                spawn_piece(&mut commands, new_piece, game_resources.clone());
            }
        }
    }
}

// #[derive(Resource, Default)]
// pub struct SelectedPiece {
//     pub entity: Option<Entity>,
// }

fn handle_piece_picking_event(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>,
    pieces_query: Query<(Entity, &Piece)>,
    game: Res<game::Game>,
    game_resources: Res<GameResources>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(e) => {
                for (entity, piece) in pieces_query.iter() {
                    if entity.index() == e.index() {
                        println!("Clicked on piece: {:?}", entity);
                        let possible_positions =
                            game.get_possible_moves_for_piece(game.player, piece.position - 1);

                        // Despawn possible candidates
                        pieces_query
                            .iter()
                            .filter(|(_, piece)| piece.candidate)
                            .for_each(|(entity, _)| {
                                commands.entity(entity).despawn();
                            });

                        // Spawn new candidates
                        for position in possible_positions.iter() {
                            println!("Possible position: {:?}", position);

                            let row = game.board.get_next_free_row(*position);
                            spawn_piece(
                                &mut commands,
                                Piece {
                                    position: *position + 1,
                                    row,
                                    color: game.player,
                                    highlighted: false,
                                    candidate: true,
                                },
                                game_resources.clone(),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
