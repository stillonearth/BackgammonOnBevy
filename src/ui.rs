use bevy::prelude::*;
use bevy_dice::*;

use std::time::Duration;

use crate::{events::DiceRollTimer, game};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub(crate) struct LabelPlayerTurn;

#[derive(Component)]
pub(crate) struct ButtonRollDice;

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

pub(crate) fn ui_logic(
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
