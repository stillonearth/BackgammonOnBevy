use bevy::prelude::*;
use bevy_dice::*;
use bevy_kira_audio::prelude::*;
use bevy_mod_picking::PickingEvent;

use crate::{
    game::{self, GameLogEntry},
    spawn_piece, spawn_pieces,
    ui::{ButtonBearOff, ButtonRollDice, LabelGameOver, LabelMoveStack, LabelPlayerTurn},
    GameResources, Piece,
};

#[derive(Default, Clone, Resource)]
pub struct HighlightPickablePiecesEvent;

#[allow(dead_code)]
#[derive(Clone, Resource)]
pub struct TurnStartEvent {
    player: game::Color,
}

#[derive(Default, Clone, Resource)]
pub struct DisplayPossibleMovesEvent {
    pub(crate) position: usize,
    pub(crate) entity: Option<Entity>,
}

#[derive(Default, Clone, Resource)]
pub struct MovePieceEvent {
    pub(crate) from: usize,
    pub(crate) to: i32,
}

#[derive(Default, Clone, Resource)]
pub struct MovePieceEndEvent;

#[derive(Clone, Resource)]
pub struct GameOverEvent {
    player: game::Color,
}

#[derive(Component)]
pub(crate) struct DiceRollTimer {
    pub(crate) timer: Timer,
}

#[derive(Default, Clone, Resource)]
pub struct StartGameEvent;

pub(crate) fn event_dice_roll_result(
    mut dice_rolls: EventReader<DiceRollResult>,
    mut game: ResMut<game::Game>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    let player = game.player;
    for event in dice_rolls.iter() {
        audio.play(asset_server.load("sounds/throw.wav"));
        game.game_log.push(GameLogEntry {
            player,
            dice_rolls: event.values[0].clone(),
        });
    }
}

pub(crate) fn event_dice_rolls_complete(
    mut commands: Commands,
    mut dice_roll_timer_query: Query<(Entity, &mut DiceRollTimer)>,
    time: Res<Time>,
    mut game: ResMut<game::Game>,
    mut turn_start_event_writer: EventWriter<TurnStartEvent>,
    mut highlight_pickable_pieces_event_writer: EventWriter<HighlightPickablePiecesEvent>,
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

            let possible_moves = game.get_possible_moves(game.player, game.dice_rolls.clone());

            if possible_moves.is_empty() {
                game.switch_turn();

                turn_start_event_writer.send(TurnStartEvent {
                    player: game.player,
                });
            } else {
                highlight_pickable_pieces_event_writer.send(HighlightPickablePiecesEvent);
            }

            break;
        }
    }
}

pub(crate) fn handle_piece_picking(
    mut picking_event_reader: EventReader<PickingEvent>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
    mut display_possible_moves_event_writer: EventWriter<DisplayPossibleMovesEvent>,
    mut move_piece_event_writer: EventWriter<MovePieceEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for event in picking_event_reader.iter() {
        if let PickingEvent::Clicked(e) = event {
            // remove selection from Piece entity

            audio.play(asset_server.load("sounds/click.wav"));

            let all_pieces = pieces_query
                .iter()
                .map(|(_, piece)| *piece)
                .collect::<Vec<_>>();

            for (entity, piece) in pieces_query.iter_mut() {
                if entity.index() == e.index() {
                    if piece.highlighted {
                        display_possible_moves_event_writer.send(DisplayPossibleMovesEvent {
                            position: piece.position,
                            entity: Some(entity),
                        });
                    }

                    if piece.candidate {
                        let chosen_piece = all_pieces.iter().find(|p| p.chosen).unwrap();
                        move_piece_event_writer.send(MovePieceEvent {
                            from: chosen_piece.position,
                            to: piece.position as i32,
                        });
                    }
                }
            }
        }
    }
}

pub(crate) fn handle_display_possible_moves(
    mut commands: Commands,
    mut display_possible_moves_event_reader: EventReader<DisplayPossibleMovesEvent>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
    mut button_bear_off_query: Query<(&mut Visibility, &mut Style, &mut ButtonBearOff)>,
    game: Res<game::Game>,
    game_resources: Res<GameResources>,
) {
    for event in display_possible_moves_event_reader.iter() {
        let possible_positions = game.get_possible_moves_for_piece(game.player, event.position - 1);

        // Despawn possible candidates
        pieces_query
            .iter()
            .filter(|(_, piece)| piece.candidate)
            .for_each(|(entity, _)| {
                commands.entity(entity).despawn();
            });

        // Set chosen piece
        pieces_query.iter_mut().for_each(|(entity, mut piece)| {
            piece.chosen = entity.index() == event.entity.unwrap().index();
        });

        for (mut visibility, mut style, mut button) in &mut button_bear_off_query.iter_mut() {
            *visibility = Visibility::Hidden;
            style.display = Display::None;
            button.position_to = None;
        }

        for position in possible_positions.iter() {
            if *position >= 24 || *position < 0 {
                for (mut visibility, mut style, mut button) in &mut button_bear_off_query.iter_mut()
                {
                    *visibility = Visibility::Inherited;
                    style.display = Display::Flex;
                    button.position_to = Some(*position + 1);
                }
                break;
            }

            // Moves on board
            let row = game.board.get_next_free_row(*position as usize);
            spawn_piece(
                &mut commands,
                Piece {
                    position: (*position + 1) as usize,
                    row,
                    color: game.player,
                    highlighted: false,
                    candidate: true,
                    chosen: false,
                },
                game_resources.clone(),
            );
        }
    }
}

pub(crate) fn handle_hightlight_choosable_pieces(
    mut commands: Commands,
    game: Res<game::Game>,
    mut query: Query<(Entity, &Piece)>,
    game_resources: Res<GameResources>,
    mut player_turn_event_choose_piece_event_reader: EventReader<HighlightPickablePiecesEvent>,
) {
    if player_turn_event_choose_piece_event_reader.iter().count() == 0 {
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
                let mut new_piece = *piece;
                new_piece.highlighted = true;
                spawn_piece(&mut commands, new_piece, game_resources.clone());
            }
        }
    }
}

pub(crate) fn handle_move_piece_event(
    mut commands: Commands,
    mut display_possible_moves_event_reader: EventReader<MovePieceEvent>,
    mut highlight_pickable_pieces_event_writer: EventWriter<HighlightPickablePiecesEvent>,
    mut move_piece_end_event_writer: EventWriter<MovePieceEndEvent>,
    pieces_query: Query<(Entity, &Piece)>,
    mut game: ResMut<game::Game>,
    game_resources: Res<GameResources>,
) {
    if display_possible_moves_event_reader.is_empty() {
        return;
    }

    for event in display_possible_moves_event_reader.iter() {
        let player = game.player;
        game.board
            .make_move(player, event.from - 1, event.to - 1)
            .unwrap();

        let move_ = (event.to - event.from as i32).unsigned_abs() as usize;
        let number_of_same_moves = game.dice_rolls.iter().filter(|&&x| x == move_).count();
        game.dice_rolls = game
            .dice_rolls
            .iter()
            .filter(|&&x| x != move_)
            .cloned()
            .collect();

        if number_of_same_moves > 1 {
            for _ in 0..number_of_same_moves - 1 {
                game.dice_rolls.push(move_);
            }
        }
    }

    if !game.dice_rolls.is_empty() {
        highlight_pickable_pieces_event_writer.send(HighlightPickablePiecesEvent);
    }

    // redraw the board
    pieces_query.iter().for_each(|(entity, _)| {
        commands.entity(entity).despawn();
    });
    spawn_pieces(commands, game, game_resources);

    move_piece_end_event_writer.send(MovePieceEndEvent);
}

pub(crate) fn handle_move_piece_end_event(
    mut move_piece_end_event_reader: EventReader<MovePieceEndEvent>,
    mut turn_start_event_writer: EventWriter<TurnStartEvent>,
    mut game_over_event_writer: EventWriter<GameOverEvent>,
    mut game: ResMut<game::Game>,
) {
    if move_piece_end_event_reader.is_empty() {
        return;
    }

    for _ in move_piece_end_event_reader.iter() {
        if game.is_over() {
            game_over_event_writer.send(GameOverEvent {
                player: game.player,
            });
            return;
        }

        if game.can_move(game.player) {
        } else {
            game.switch_turn();

            turn_start_event_writer.send(TurnStartEvent {
                player: game.player,
            });
        }
    }
}

pub(crate) fn handle_dice_roll_start_event(
    mut dice_roll_start_event_reader: EventReader<DiceRollStartEvent>,
    mut query_button_roll_dice: Query<&mut Visibility, With<ButtonRollDice>>,
) {
    for _ in dice_roll_start_event_reader.iter() {
        for mut visibility in query_button_roll_dice.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(crate) fn handle_turn_start_event(
    mut turn_start_event_reader: EventReader<TurnStartEvent>,
    mut query_button_roll_dice: Query<&mut Visibility, With<ButtonRollDice>>,
    _game: ResMut<game::Game>,
) {
    for _ in turn_start_event_reader.iter() {
        for mut visibility in query_button_roll_dice.iter_mut() {
            *visibility = Visibility::Inherited;
        }
    }
}

pub(crate) fn handle_game_over_event(
    mut event_game_over_reader: EventReader<GameOverEvent>,
    mut ui_elements_param_set: ParamSet<(
        Query<(&mut Visibility, With<ButtonRollDice>)>,
        Query<(&mut Visibility, With<ButtonBearOff>)>,
        Query<(&mut Visibility, With<LabelPlayerTurn>)>,
        Query<(&mut Visibility, With<LabelMoveStack>)>,
        Query<(&mut Text, &mut Visibility, With<LabelGameOver>)>,
    )>,
) {
    for e in event_game_over_reader.iter() {
        for (mut v, _) in ui_elements_param_set.p0().iter_mut() {
            *v = Visibility::Hidden;
        }

        for (mut v, _) in ui_elements_param_set.p1().iter_mut() {
            *v = Visibility::Hidden;
        }

        for (mut v, _) in ui_elements_param_set.p2().iter_mut() {
            *v = Visibility::Hidden;
        }

        for (mut v, _) in ui_elements_param_set.p3().iter_mut() {
            *v = Visibility::Hidden;
        }

        for (mut text, mut v, _) in ui_elements_param_set.p4().iter_mut() {
            *v = Visibility::Inherited;
            text.sections[0].value = format!("{:?} Won!", e.player);
            text.sections[0].style.color = match e.player {
                game::Color::White => Color::WHITE,
                game::Color::Black => Color::BLACK,
            };
        }
    }
}

pub(crate) fn handle_start_game_event(
    mut start_game_event_reader: EventReader<StartGameEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for _ in start_game_event_reader.iter() {
        let sound = asset_server.load("sounds/background.mp3");
        audio.play(sound).looped();
    }
}
