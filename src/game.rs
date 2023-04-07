use bevy::prelude::Resource;
use itertools::Itertools;
use std::ops::Range;

// Define the type of game piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

// Define the type of game board.
pub struct Board {
    pub points: [i32; 24], // Number of pieces on each point of the board.
    pub bar: [i32; 2],     // Number of pieces on the bar.
}

impl Board {
    pub(crate) fn is_player_home_complete(&self, color: Color) -> bool {
        let mut home_board = if color == Color::White { 18..24 } else { 0..6 };
        let home_of_same_color = home_board.all(|i| {
            let clr = self.get_point_color(i as usize);
            clr.is_none() || clr.unwrap() == color
        });

        let mut rest_of_board = if color == Color::White { 0..18 } else { 6..24 };
        let rest_of_board_is_empty = rest_of_board.all(|i| {
            let clr = self.get_point_color(i as usize);
            clr.is_none() || clr.unwrap() != color
        });
        home_of_same_color && rest_of_board_is_empty
    }

    pub fn make_move(
        &mut self,
        player: Color,
        from_position: usize,
        to_position: i32,
    ) -> Result<(), String> {
        // check if move is valid
        if !self.can_move_piece(player, from_position, to_position) {
            return Err(String::from("Invalid move"));
        }

        let direction = self.direction(player);
        self.points[from_position] -= direction;

        let is_home_complete = self.is_player_home_complete(player);
        if is_home_complete && player == Color::White && to_position >= 24 {
            return Ok(());
        }

        if is_home_complete && player == Color::Black && to_position < 0 {
            return Ok(());
        }

        let to_position = to_position as usize;

        if self.points[to_position] == -direction {
            self.points[to_position] = direction;
            self.bar[self.opposite_bar_index(player)] += 1;
        } else {
            self.points[to_position] += direction;
        }

        Ok(())
    }

    pub fn can_move_piece(&self, player: Color, from_point: usize, to_point: i32) -> bool {
        if self.get_point_count(from_point) == 0 {
            return false;
        }

        if self.get_point_color(from_point) != Some(player) {
            return false;
        }

        let is_home_complete = self.is_player_home_complete(player);
        if is_home_complete && player == Color::White && to_point >= 23 {
            return true;
        }
        if is_home_complete && player == Color::Black && to_point < 0 {
            return true;
        }

        if to_point < 0 || to_point >= 24 {
            return false;
        }

        let opposite_color = player.opposite();
        let to_point_color = self.get_point_color(to_point as usize);
        let to_point_count = self.get_point_count(to_point as usize);

        if to_point_color == Some(opposite_color) && to_point_count > 0 {
            return false;
        }

        let to_point: usize = to_point as usize;

        let direction = if player == Color::White { 1 } else { -1 };
        if to_point >= 24 {
            return false;
        }
        if to_point_color == Some(opposite_color) && to_point < from_point && direction == 1 {
            return false;
        }
        if to_point_color == Some(opposite_color) && to_point > from_point && direction == -1 {
            return false;
        }

        if self.points[to_point].abs() >= 5 {
            return false;
        }

        true
    }

    fn get_point_color(&self, point: usize) -> Option<Color> {
        let point_count = self.points[point];

        match point_count {
            0 => None,
            _ if point_count > 0 => Some(Color::White),
            _ => Some(Color::Black),
        }
    }

    fn get_point_count(&self, point: usize) -> usize {
        self.points[point].unsigned_abs() as usize
    }

    pub fn opposite_bar_index(&self, color: Color) -> usize {
        match color {
            Color::White => 1,
            Color::Black => 0,
        }
    }

    fn get_index(&self, color: Color, index: usize, dice_roll_value: usize) -> i32 {
        match color {
            Color::White => index as i32 + dice_roll_value as i32,
            Color::Black => index as i32 - dice_roll_value as i32,
        }
    }

    #[allow(dead_code)]
    fn home(&self, player: Color) -> Range<usize> {
        if player == Color::White {
            18..24
        } else {
            0..6
        }
    }

    fn get_points_for_color(&self, color: Color) -> Vec<usize> {
        let mut points = vec![];
        for i in 0..24 {
            if self.get_point_color(i) == Some(color) {
                points.push(i);
            }
        }
        points
    }

    fn direction(&self, player: Color) -> i32 {
        if player == Color::White {
            1
        } else {
            -1
        }
    }

    pub fn get_next_free_row(&self, position: usize) -> usize {
        self.points[position].unsigned_abs() as usize + 1
    }
}

#[derive(Clone)]
pub struct GameLogEntry {
    pub player: Color,
    pub dice_rolls: Vec<usize>,
}

#[derive(Resource)]
pub(crate) struct Game {
    pub board: Board,
    pub dice_rolls: Vec<usize>,
    pub dice_rolled: bool,
    pub player: Color,
    pub game_log: Vec<GameLogEntry>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub(crate) fn can_move(&self, player: Color) -> bool {
        let possible_moves = self.get_possible_moves(player, self.dice_rolls.clone());
        !possible_moves.is_empty()
    }

    pub(crate) fn get_possible_moves(
        &self,
        player: Color,
        dice_rolls: Vec<usize>,
    ) -> Vec<(usize, i32)> {
        let mut moves: Vec<(usize, i32)> = vec![];
        let indices = self.board.get_points_for_color(player);

        for &index in indices.iter() {
            for dice_roll in dice_rolls.iter() {
                let next_index = self.board.get_index(player, index, *dice_roll);
                if self.board.can_move_piece(player, index, next_index) {
                    moves.push((index, next_index));
                }
            }
        }

        moves
    }

    pub(crate) fn get_possible_moves_for_piece(&self, player: Color, piece: usize) -> Vec<i32> {
        let unique_rolls: Vec<usize> = self
            .dice_rolls
            .clone()
            .iter()
            .unique()
            .copied()
            .collect_vec();

        let possible_moves = self.get_possible_moves(player, unique_rolls);

        let mut possible_moves: Vec<i32> = possible_moves
            .iter()
            .filter(|(from, _)| *from == piece)
            .map(|(_, to)| *to)
            .collect();

        possible_moves.sort();
        if player == Color::Black {
            possible_moves.reverse();
        }
        return possible_moves;
    }

    pub(crate) fn get_choosable_pieces(&self) -> (Vec<[usize; 2]>, [usize; 2]) {
        let mut choosable_pieces_on_board: Vec<[usize; 2]> = vec![];
        let choosable_bar_pieces = [0, 0];

        let possible_moves = self.get_possible_moves(self.player, self.dice_rolls.clone());

        // fill choosable_pieces_on_board with pieces that can be chosen according to their color (value)
        for i in 0..24 {
            let point_count = self.board.points[i];

            if point_count == 0 {
                continue;
            }

            if point_count < 0 && self.player != Color::Black {
                continue;
            }

            if point_count > 0 && self.player != Color::White {
                continue;
            }

            let position_in_possible_moveset =
                possible_moves.iter().filter(|(from, _)| *from == i).count();

            if position_in_possible_moveset == 0 {
                continue;
            }

            choosable_pieces_on_board.push([i + 1, point_count.unsigned_abs() as usize]);
        }

        (choosable_pieces_on_board, choosable_bar_pieces)
    }

    pub(crate) fn new() -> Self {
        let mut points = [0; 24];
        points[0] = 2;
        points[23] = -2;
        points[5] = -5;
        points[18] = 5;
        points[16] = 3;
        points[7] = -3;
        points[12] = -5;
        points[11] = 5;

        // points[18] = 5;
        // points[19] = 5;
        // points[20] = 5;

        // points[0] -= 5;
        // points[1] -= 5;
        // points[2] -= 5;

        // Create a new game instance with an empty board and start with player 1
        Game {
            board: Board {
                points,
                bar: [0, 0],
            },
            dice_rolls: vec![],
            player: Color::White,
            dice_rolled: false,
            game_log: vec![],
        }
    }

    pub(crate) fn switch_turn(&mut self) {
        self.player = self.player.opposite();
        self.dice_rolled = false;
        self.dice_rolls = vec![];
    }

    pub(crate) fn highest_point_in_home_zone(&self) -> (usize, i32) {
        let mut highest_index = 0;
        let mut highest_value = 0;
        for i in self.board.home(self.player) {
            if self.board.points[i] > highest_value * self.board.direction(self.player) {
                highest_index = i;
                highest_value = self.board.points[i];
            }
        }

        (highest_index, highest_value)
    }

    #[allow(dead_code)]
    pub(crate) fn bear_off_piece(&mut self, from: i32, roll: i32) {
        let direction = if self.player == Color::White { 1 } else { -1 };
        let index = from - direction;
        let value = self.board.points[index as usize];

        if value == 0 {
            // special case
            let next_index = (index - direction) as usize;
            let next_destination = self.board.get_index(self.player, next_index, roll as usize);
            if self
                .board
                .can_move_piece(self.player, next_index, next_destination)
            {
                self.board
                    .make_move(self.player, next_index, next_destination)
                    .unwrap();
            } else {
                // remove a piece from the highest point on which one of this checkers resides
                let (highest_index, _highest_value) = self.highest_point_in_home_zone();
                self.board.points[highest_index] -= direction;
            }
        } else {
            self.board.points[index as usize] -= direction;
        }
    }

    pub(crate) fn is_over(&self) -> bool {
        let player1_borne_off = self.board.points[0..18].iter().all(|&x| x == 0);
        let player2_borne_off = self.board.points[6..24].iter().all(|&x| x == 0);
        player1_borne_off || player2_borne_off
    }
}
