use bevy::prelude::*;
use bevy_dice::*;
use bevy_mod_picking::PickingEvent;

use crate::{
    game::{self, GameLogEntry},
    spawn_piece, spawn_pieces, GameResources, Piece,
};

#[derive(Default, Clone, Resource)]
pub struct HighlightPickablePiecesEvent;

#[derive(Default, Clone, Resource)]
pub struct DisplayPossibleMovesEvent {
    pub(crate) position: usize,
    pub(crate) entity: Option<Entity>,
}

#[derive(Default, Clone, Resource)]
pub struct MovePieceEvent {
    pub(crate) from: usize,
    pub(crate) to: usize,
}

#[derive(Component)]
pub(crate) struct DiceRollTimer {
    pub(crate) timer: Timer,
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

pub(crate) fn event_dice_rolls_complete(
    mut commands: Commands,
    mut dice_roll_timer_query: Query<(Entity, &mut DiceRollTimer)>,
    time: Res<Time>,
    mut game: ResMut<game::Game>,
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
            highlight_pickable_pieces_event_writer.send(HighlightPickablePiecesEvent);
            break;
        }
    }
}

pub(crate) fn handle_piece_picking(
    _commands: Commands,
    mut picking_event_reader: EventReader<PickingEvent>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
    mut display_possible_moves_event_writer: EventWriter<DisplayPossibleMovesEvent>,
    mut move_piece_event_writer: EventWriter<MovePieceEvent>,
) {
    for event in picking_event_reader.iter() {
        if let PickingEvent::Clicked(e) = event {
            // remove selection from Piece entity

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
                            to: piece.position,
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

        // Spawn new candidates
        for position in possible_positions.iter() {
            let row = game.board.get_next_free_row(*position);
            spawn_piece(
                &mut commands,
                Piece {
                    position: *position + 1,
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

#[allow(unused_variables)]
pub(crate) fn handle_move_piece_event(
    mut commands: Commands,
    mut display_possible_moves_event_reader: EventReader<MovePieceEvent>,
    mut highlight_pickable_pieces_event_writer: EventWriter<HighlightPickablePiecesEvent>,
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

        let move_ = (event.to as i32 - event.from as i32).abs() as usize;
        let number_of_same_moves = game.dice_rolls.iter().filter(|&&x| x == move_).count();
        game.dice_rolls = game
            .dice_rolls
            .iter()
            .filter(|&&x| x != move_)
            .cloned()
            .collect();

        for i in 0..number_of_same_moves - 1 {
            game.dice_rolls.push(move_);
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
}
