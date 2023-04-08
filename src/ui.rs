use bevy::prelude::*;
use bevy_dice::*;

use std::time::Duration;

use crate::{
    events::{DiceRollTimer, MovePieceEvent},
    game, Piece,
};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub(crate) struct LabelPlayerTurn;

#[derive(Component)]
pub(crate) struct LabelGameOver;

#[derive(Component)]
pub(crate) struct ButtonRollDice;

#[derive(Component)]
pub(crate) struct ButtonBearOff {
    pub(crate) position_to: Option<i32>,
}

#[derive(Component)]
pub(crate) struct LabelMoveStack;

pub(crate) fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
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
                        font_size: 120.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ))
                .insert(LabelGameOver);
        })
        .insert(Name::new("GameOver"));

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

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        display: Display::None,
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Bear Off",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                })
                .insert(ButtonBearOff { position_to: None });
        })
        .insert(Name::new("BottomBar"));
}

pub(crate) fn ui_logic(
    mut commands: Commands,
    pieces_query: Query<(Entity, &Piece)>,
    mut button_param_set: ParamSet<(
        Query<
            (Entity, &Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<ButtonRollDice>),
        >,
        Query<
            (
                Entity,
                &Interaction,
                &mut BackgroundColor,
                &mut Visibility,
                &ButtonBearOff,
            ),
            (Changed<Interaction>, With<ButtonBearOff>),
        >,
    )>,
    mut label_set: ParamSet<(
        Query<&mut Text, With<LabelPlayerTurn>>,
        Query<&mut Text, With<LabelMoveStack>>,
    )>,
    mut dice_roll_start_event_writer: EventWriter<DiceRollStartEvent>,
    mut move_piece_event_writer: EventWriter<MovePieceEvent>,
    mut game: ResMut<game::Game>,
) {
    for (_entity, interaction, mut color) in &mut button_param_set.p0() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();

                let num_dice: Vec<usize> = vec![2, 2];

                dice_roll_start_event_writer.send(DiceRollStartEvent { num_dice });
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

    for (_entity, interaction, mut color, mut visibility, button_bear_off) in
        &mut button_param_set.p1()
    {
        match *interaction {
            Interaction::Clicked => {
                let all_pieces = pieces_query
                    .iter()
                    .map(|(_, piece)| *piece)
                    .collect::<Vec<_>>();
                let chosen_piece = all_pieces.iter().find(|p| p.chosen);

                if chosen_piece.is_none() {
                    continue;
                }

                move_piece_event_writer.send(MovePieceEvent {
                    from: chosen_piece.unwrap().position,
                    to: button_bear_off.position_to.unwrap(),
                });

                *visibility = Visibility::Hidden;
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

        if game.board.is_player_home_complete(game.player) {
            text.sections[0].value = format!("{} \t Bear Off!", text.sections[0].value);
        }
    }

    for mut text in &mut label_set.p1().iter_mut() {
        if !game.dice_rolls.is_empty() {
            text.sections[0].value = format!("Move Stack: {:?}", game.dice_rolls);
        } else {
            text.sections[0].value = "".to_string();
        }
    }
}
