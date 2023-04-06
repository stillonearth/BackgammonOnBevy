use bevy::prelude::*;
use bevy_dice::*;
use bevy_mod_picking::PickingEvent;

use crate::{
    game::{self, GameLogEntry},
    spawn_piece, GameResources, GameUIState, Piece,
};

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

pub(crate) fn handle_piece_picking_event(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>,
    pieces_query: Query<(Entity, &Piece)>,
    game: Res<game::Game>,
    game_resources: Res<GameResources>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            for (entity, piece) in pieces_query.iter() {
                if entity.index() == e.index() {
                    println!("Clicked on piece: {entity:?}");
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
                        println!("Possible position: {position:?}");

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
    }
}
