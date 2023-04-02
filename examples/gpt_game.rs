use rand::Rng;
use std::{io, ops::Range};

// Define the type of dice.
enum Dice {
    D6,
}

// Define the type of game piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Color {
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
struct Board {
    points: [i32; 24], // Number of pieces on each point of the board.
    bar: [i32; 2],     // Number of pieces on the bar.
}

impl Board {
    fn is_player_home_complete(&self, color: Color) -> bool {
        let home_board = if color == Color::White { 18..24 } else { 0..6 };
        let home_of_same_color = home_board.clone().all(|i| {
            let clr = self.get_point_color(i as usize);
            return clr.is_none() || clr.unwrap() == color;
        });

        let rest_of_board = if color == Color::White { 0..18 } else { 6..24 };
        let rest_of_board_is_empty = rest_of_board.clone().all(|i| {
            let clr = self.get_point_color(i as usize);
            return clr.is_none() || clr.unwrap() != color;
        });
        return home_of_same_color && rest_of_board_is_empty;
    }

    fn print(&self) {
        let points = self.points;

        println!("| 13|14|15|16|17|18|   |19|20|21|22|23|24 |");
        println!("|------------------|   |------------------|");
        for row in 1..=5 {
            print!("|");
            for point in 13..=24 {
                if points[point - 1] >= row {
                    print!(" W ");
                } else if points[point - 1] <= -row {
                    print!(" B ");
                } else {
                    print!("   ");
                }

                if point == 18 {
                    print!("|   |");
                }

                if point == 24 {
                    println!("|");
                }
            }
        }

        println!("|------------------|   |------------------|");

        for row in (1..=5).rev() {
            print!("|");
            for point in (1..=12).rev() {
                if points[point - 1] >= row {
                    print!(" W ");
                } else if points[point - 1] <= -row {
                    print!(" B ");
                } else {
                    print!("   ");
                }

                if point == 7 {
                    print!("|   |");
                }

                if point == 1 {
                    println!("|");
                }
            }
        }
        println!("|------------------|   |------------------|");
        println!("| 12|11|10| 9| 8| 7|   | 6| 5| 4| 4| 2| 1 |");
    }

    pub fn make_move(
        &mut self,
        player: Color,
        from_point: usize,
        to_point: usize,
    ) -> Result<(), String> {
        // check if move is valid
        if !self.can_move_piece(player, from_point, to_point) {
            return Err(String::from("Invalid move"));
        }

        let direction = self.direction(player);
        self.points[from_point] -= direction;

        if self.points[to_point] == -direction {
            self.points[to_point] = direction;
            self.bar[self.opposite_bar_index(player)] += 1;
        } else {
            self.points[to_point] += direction;
        }

        return Ok(());
    }

    pub fn can_move_piece(&self, player: Color, from_point: usize, to_point: usize) -> bool {
        // Проверяем, что на точке from_point есть хотя бы одна фишка
        if self.get_point_count(from_point) == 0 {
            return false;
        }

        // Проверяем, что фишка нужного цвета находится на точке from_point
        if self.get_point_color(from_point) != Some(player) {
            return false;
        }

        // Проверяем, что на точке to_point нет более одной фишки противоположного цвета
        let opposite_color = player.opposite();
        let to_point_color = self.get_point_color(to_point);
        let to_point_count = self.get_point_count(to_point);

        if to_point_color == Some(opposite_color) && to_point_count > 1 {
            return false;
        }

        // Проверяем, что точка назначения находится в допустимой зоне для хода
        let direction = if player == Color::White { 1 } else { -1 };
        if to_point < 0 || to_point >= 24 || to_point == 0 || to_point == 23 {
            return false;
        }
        if to_point_color == Some(opposite_color) && to_point < from_point && direction == 1 {
            return false;
        }
        if to_point_color == Some(opposite_color) && to_point > from_point && direction == -1 {
            return false;
        }

        true
    }

    fn get_point_color(&self, point: usize) -> Option<Color> {
        let point_count = self.points[point];
        if point_count == 0 {
            None
        } else if point_count > 0 {
            Some(Color::White)
        } else {
            Some(Color::Black)
        }
    }

    fn get_point_count(&self, point: usize) -> usize {
        self.points[point].abs() as usize
    }

    pub fn opposite_bar_index(&self, color: Color) -> usize {
        match color {
            Color::White => 1,
            Color::Black => 0,
        }
    }

    fn get_index(&self, color: Color, index: usize, dice_roll_value: usize) -> usize {
        let mut idx = match color {
            Color::White => index + dice_roll_value,
            Color::Black => index.saturating_sub(dice_roll_value),
        };

        if idx >= 24 {
            idx -= 24;
        }

        return idx;
    }

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
}

// Определяем тип игры.
struct Game {
    board: Board,
    dice: [i32; 2], // Результаты бросков костей.
    player: Color,  // Цвет игрока, который должен сделать ход.
}

impl Game {
    pub fn can_move(&self, player: Color) -> bool {
        return false;
    }

    pub fn get_possible_moves(&self) -> Vec<(usize, usize)> {
        let mut moves: Vec<(usize, usize)> = vec![];
        let indices = self.board.get_points_for_color(self.player);

        for &index in indices.iter() {
            let next_index_dice_1 = self
                .board
                .get_index(self.player, index, self.dice[0] as usize);
            let next_index_dice_2 = self
                .board
                .get_index(self.player, index, self.dice[1] as usize);

            if self
                .board
                .can_move_piece(self.player, index, next_index_dice_1)
            {
                moves.push((index, next_index_dice_1));
            }

            if self
                .board
                .can_move_piece(self.player, index, next_index_dice_1)
            {
                moves.push((index, next_index_dice_2));
            }
        }

        moves
    }

    fn get_player_move(&self) -> Option<(usize, usize)> {
        let possible_moves = self.get_possible_moves();
        let mut valid_move = false;
        let mut from_index = 0;
        let mut to_index = 0;

        while !valid_move {
            println!(
                "Available moves for {:?} with dice roll {:?}: {:?}",
                self.player, self.dice, possible_moves
            );
            println!("Enter the index of the piece you want to move:");

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            match input.trim().parse::<usize>() {
                Ok(index) if possible_moves.iter().any(|(i, _)| *i == index) => {
                    from_index = index;
                    println!("Enter the index to move the piece to:");

                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");

                    match input.trim().parse::<usize>() {
                        Ok(index) if possible_moves.iter().any(|(_, j)| *j == index) => {
                            to_index = index;
                            valid_move = true;
                        }
                        Ok(index) => {
                            println!("Invalid move destination: {}", index);
                        }
                        Err(_) => {
                            println!("Invalid input, please enter a number");
                        }
                    }
                }
                Ok(index) => {
                    println!("Invalid piece index: {}", index);
                }
                Err(_) => {
                    println!("Invalid input, please enter a number");
                }
            }
        }

        Some((from_index, to_index))
    }

    fn roll_dice(&mut self) {
        self.dice[0] = rand::thread_rng().gen_range(1..=6);
        self.dice[1] = rand::thread_rng().gen_range(1..=6);
    }

    fn new() -> Self {
        let mut points = [0; 24];
        points[0] = 2;
        points[23] = -2;
        points[5] = -5;
        points[18] = 5;
        points[16] = 3;
        points[7] = -3;
        points[12] = -5;
        points[11] = 5;

        // Create a new game instance with an empty board and start with player 1
        Game {
            board: Board {
                points,
                bar: [0, 0],
            },
            dice: [0, 0],
            player: Color::White,
        }
    }

    fn get_winner(&self) -> Option<Color> {
        if self.board.points[18..24].iter().all(|&x| x == 0) {
            Some(Color::White)
        } else if self.board.points[0..6].iter().all(|&x| x == 0) {
            Some(Color::Black)
        } else {
            None
        }
    }

    fn switch_turn(&mut self) {
        self.player = self.player.opposite();
    }

    fn highest_point_in_home_zone(&self) -> (usize, i32) {
        let mut highest_index = 0;
        let mut highest_value = 0;
        for i in self.board.home(self.player) {
            if self.board.points[i] > highest_value * self.board.direction(self.player) {
                highest_index = i;
                highest_value = self.board.points[i];
            }
        }

        return (highest_index, highest_value);
    }

    fn bear_off(&self) {}

    fn bear_off_piece(&mut self, from: i32, roll: i32) {
        let direction = if self.player == Color::White { 1 } else { -1 };
        let index = from - direction;
        let value = self.board.points[index as usize];

        if value == 0 {
            // special case
            let next_index = (index - direction) as usize;
            let next_destination =
                self.board
                    .get_index(self.player, next_index as usize, roll as usize);
            if self
                .board
                .can_move_piece(self.player, next_index, next_destination)
            {
                self.board
                    .make_move(self.player, next_index, next_destination)
                    .unwrap();
            } else {
                // remove a piece from the highest point on which one of this checkers resides
                let (highest_index, highest_value) = self.highest_point_in_home_zone();
                self.board.points[highest_index] -= direction;
            }
        } else {
            self.board.points[index as usize] -= direction;
        }
    }

    fn is_over(&self) -> bool {
        let player1_borne_off = self.board.points[0..18].iter().all(|&x| x == 0);
        let player2_borne_off = self.board.points[6..24].iter().all(|&x| x == 0);
        player1_borne_off || player2_borne_off
    }
}

fn play_game() {
    // Create a new game instance
    let mut game = Game::new();

    // Start the main game loop
    while !game.is_over() {
        // Print the current board state and player turn
        game.board.print();
        println!("It's {:?}'s turn", game.player);

        game.roll_dice();
        // Check if the current player can move any pieces
        if game.can_move(game.player) {
            // Bear off any pieces that have reached the end of the home board
            if game.board.is_player_home_complete(game.player) {
                game.bear_off();
                return;
            }

            // Get the player's move
            let player_move = game.get_player_move();

            if let Some((from, to)) = player_move {
                game.board.make_move(game.player, from, to);
            }

            // Make the move
        } else {
            println!("{:?} can't move, switching turn", game.player);

            // Switch the turn to the other player
            game.switch_turn();
        }
    }

    // Print the winner
    println!("Game over! {:?} wins", game.get_winner());
}

fn main() {
    // play_game();

    let mut game = Game::new();
    game.board.print();
}
