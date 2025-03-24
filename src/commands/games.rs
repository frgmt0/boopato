use crate::CommandError;
use poise::serenity_prelude as serenity;
use std::fmt;
use std::time::{Duration, Instant};
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Tic-Tac-Toe Game
#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    X,
    O,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, "⬜"),
            Cell::X => write!(f, "❌"),
            Cell::O => write!(f, "⭕"),
        }
    }
}

struct TicTacToe {
    board: [Cell; 9],
    current_player: Cell,
    opponent: Option<serenity::UserId>,
    ai_mode: bool,
    game_over: bool,
    winner: Option<Cell>,
}

impl TicTacToe {
    fn new(_starter: serenity::UserId, opponent: Option<serenity::UserId>) -> Self {
        let ai_mode = opponent.is_none();
        TicTacToe {
            board: [Cell::Empty; 9],
            current_player: Cell::X, // X always starts
            opponent,
            ai_mode,
            game_over: false,
            winner: None,
        }
    }

    fn is_valid_move(&self, position: usize) -> bool {
        position < 9 && self.board[position] == Cell::Empty
    }

    fn make_move(&mut self, position: usize) -> bool {
        if !self.is_valid_move(position) || self.game_over {
            return false;
        }
        
        self.board[position] = self.current_player;
        
        // Check for win or draw
        if self.check_winner() {
            self.game_over = true;
            self.winner = Some(self.current_player);
            return true;
        }
        
        if self.is_board_full() {
            self.game_over = true;
            return true;
        }
        
        // Switch player
        self.current_player = if self.current_player == Cell::X { Cell::O } else { Cell::X };
        true
    }
    
    fn ai_move(&mut self) -> Option<usize> {
        if self.game_over {
            return None;
        }
        
        // Check if AI can win in the next move
        for i in 0..9 {
            if self.is_valid_move(i) {
                self.board[i] = self.current_player;
                let win = self.check_winner();
                self.board[i] = Cell::Empty; // Undo move
                
                if win {
                    self.make_move(i);
                    return Some(i);
                }
            }
        }
        
        // Check if player can win in the next move and block
        let opponent = if self.current_player == Cell::X { Cell::O } else { Cell::X };
        for i in 0..9 {
            if self.is_valid_move(i) {
                self.board[i] = opponent;
                let win = self.check_winner();
                self.board[i] = Cell::Empty; // Undo move
                
                if win {
                    self.make_move(i);
                    return Some(i);
                }
            }
        }
        
        // Take center if available
        if self.is_valid_move(4) {
            self.make_move(4);
            return Some(4);
        }
        
        // Take a corner if available
        let corners = [0, 2, 6, 8];
        let available_corners: Vec<usize> = corners.iter()
            .filter(|&&i| self.is_valid_move(i))
            .cloned()
            .collect();
            
        if !available_corners.is_empty() {
            let idx = rand::thread_rng().gen_range(0..available_corners.len());
            let corner = available_corners[idx];
            self.make_move(corner);
            return Some(corner);
        }
        
        // Take any available side
        let sides = [1, 3, 5, 7];
        let available_sides: Vec<usize> = sides.iter()
            .filter(|&&i| self.is_valid_move(i))
            .cloned()
            .collect();
            
        if !available_sides.is_empty() {
            let idx = rand::thread_rng().gen_range(0..available_sides.len());
            let side = available_sides[idx];
            self.make_move(side);
            return Some(side);
        }
        
        // No move found (should never happen unless board is full)
        None
    }
    
    fn check_winner(&self) -> bool {
        // Check rows
        for i in 0..3 {
            if self.board[i*3] != Cell::Empty && 
               self.board[i*3] == self.board[i*3 + 1] && 
               self.board[i*3] == self.board[i*3 + 2] {
                return true;
            }
        }
        
        // Check columns
        for i in 0..3 {
            if self.board[i] != Cell::Empty && 
               self.board[i] == self.board[i + 3] && 
               self.board[i] == self.board[i + 6] {
                return true;
            }
        }
        
        // Check diagonals
        if self.board[0] != Cell::Empty && 
           self.board[0] == self.board[4] && 
           self.board[0] == self.board[8] {
            return true;
        }
        
        if self.board[2] != Cell::Empty && 
           self.board[2] == self.board[4] && 
           self.board[2] == self.board[6] {
            return true;
        }
        
        false
    }
    
    fn is_board_full(&self) -> bool {
        self.board.iter().all(|&cell| cell != Cell::Empty)
    }
    
    fn render_board(&self) -> String {
        let cell_display = |i: usize| {
            format!("{}", self.board[i])
        };
        
        format!(
            "{} {} {}\n{} {} {}\n{} {} {}",
            cell_display(0), cell_display(1), cell_display(2),
            cell_display(3), cell_display(4), cell_display(5),
            cell_display(6), cell_display(7), cell_display(8)
        )
    }
    
    fn render_status(&self, player1: &serenity::User, player2: Option<&serenity::User>) -> String {
        if self.game_over {
            if let Some(winner) = self.winner {
                match winner {
                    Cell::X => format!("**{}** (❌) has won the game!", player1.name),
                    Cell::O => {
                        if let Some(p2) = player2 {
                            format!("**{}** (⭕) has won the game!", p2.name)
                        } else {
                            "The computer (⭕) has won the game!".to_string()
                        }
                    },
                    _ => "Unknown winner".to_string()
                }
            } else {
                "The game ended in a draw!".to_string()
            }
        } else {
            match self.current_player {
                Cell::X => format!("It's **{}'s** (❌) turn", player1.name),
                Cell::O => {
                    if let Some(p2) = player2 {
                        format!("It's **{}'s** (⭕) turn", p2.name)
                    } else {
                        "It's the computer's (⭕) turn".to_string()
                    }
                },
                _ => "Unknown player's turn".to_string()
            }
        }
    }
}

// Connect 4 Game - People's Revolution Edition
#[derive(Clone, Copy, PartialEq, Eq)]
enum Connect4Cell {
    Empty,
    Red,    // Player 1
    Yellow, // Player 2
}

impl std::fmt::Display for Connect4Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Connect4Cell::Empty => write!(f, "⚪"),
            Connect4Cell::Red => write!(f, "🔴"),
            Connect4Cell::Yellow => write!(f, "🟡"),
        }
    }
}

#[derive(Clone)]
struct Connect4 {
    // Standard Connect 4 has 7 columns and 6 rows
    board: [[Connect4Cell; 7]; 6],
    current_player: Connect4Cell,
    opponent: Option<serenity::UserId>,
    ai_mode: bool,
    game_over: bool,
    winner: Option<Connect4Cell>,
    last_move: Option<(usize, usize)>, // (row, col) of last move
    transposition_table: Arc<Mutex<HashMap<u64, (i32, i32)>>>, // (board_hash) -> (score, depth)
}

impl Connect4 {
    fn new(_starter: serenity::UserId, opponent: Option<serenity::UserId>) -> Self {
        let ai_mode = opponent.is_none();
        Connect4 {
            board: [[Connect4Cell::Empty; 7]; 6],
            current_player: Connect4Cell::Red, // Red always starts
            opponent,
            ai_mode,
            game_over: false,
            winner: None,
            last_move: None,
            transposition_table: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn is_valid_move(&self, column: usize) -> bool {
        if column >= 7 {
            return false;
        }
        
        // Check if the top cell in the column is empty
        self.board[0][column] == Connect4Cell::Empty
    }

    fn make_move(&mut self, column: usize) -> bool {
        if !self.is_valid_move(column) || self.game_over {
            return false;
        }
        
        // Find the lowest empty cell in the selected column
        let mut row = 5;
        while row > 0 && self.board[row][column] != Connect4Cell::Empty {
            row -= 1;
        }
        
        // If the whole column is full, this shouldn't happen due to is_valid_move check
        if self.board[row][column] != Connect4Cell::Empty {
            return false;
        }
        
        // Place the piece
        self.board[row][column] = self.current_player;
        self.last_move = Some((row, column));
        
        // Check for win or draw
        if self.check_winner() {
            self.game_over = true;
            self.winner = Some(self.current_player);
            return true;
        }
        
        if self.is_board_full() {
            self.game_over = true;
            return true;
        }
        
        // Switch player
        self.current_player = if self.current_player == Connect4Cell::Red { 
            Connect4Cell::Yellow 
        } else { 
            Connect4Cell::Red 
        };
        
        true
    }
    
    fn ai_move(&mut self) -> Option<usize> {
        if self.game_over {
            return None;
        }

        // Check opening book
        if let Some(opening_move) = self.get_opening_move() {
            if self.is_valid_move(opening_move) {
                self.make_move(opening_move);
                return Some(opening_move);
            }
        }

        // First, check for immediate winning move (depth 1)
        if let Some(winning_col) = self.find_winning_move(Connect4Cell::Yellow) {
            self.make_move(winning_col);
            return Some(winning_col);
        }
        
        // Then, check if player has a winning move and block it (depth 1)
        if let Some(blocking_col) = self.find_winning_move(Connect4Cell::Red) {
            self.make_move(blocking_col);
            return Some(blocking_col);
        }
        
        // Check for forced win in 2 moves (double threat)
        if let Some(forced_win_col) = self.find_forced_win_in_two() {
            self.make_move(forced_win_col);
            return Some(forced_win_col);
        }
        
        // Check for trap setups (creating double threats)
        if let Some(trap_col) = self.find_trap_setup() {
            self.make_move(trap_col);
            return Some(trap_col);
        }

        // Use minimax with deeper search when fewer pieces are on the board
        // and shallower search for endgame to maintain performance
        let piece_count = self.count_pieces();
        let max_depth = if piece_count < 10 {
            7 // Deep search early game
        } else if piece_count < 20 {
            6 // Medium depth mid-game
        } else {
            5 // Shallower for endgame
        };
        
        // Use iterative deepening for more responsive AI
        let mut best_col = 3; // Default to center column
        let mut best_score = i32::MIN;
        
        // Start with depth 1 and increase gradually
        for depth in 1..=max_depth {
            let (col, score) = self.minimax(depth, true, i32::MIN, i32::MAX);
            
            // If we find a winning move, use it immediately
            if score > 900000 {
                best_col = col;
                break;
            }
            
            // Otherwise update our best move
            if col >= 0 && score > best_score {
                best_col = col;
                best_score = score;
            }
        }
        
        if best_col >= 0 && best_col < 7 && self.is_valid_move(best_col as usize) {
            self.make_move(best_col as usize);
            return Some(best_col as usize);
        }
        
        // Fallback to simple strategy if minimax fails (shouldn't happen)
        if self.is_valid_move(3) {
            self.make_move(3);
            return Some(3);
        }
        
        // Otherwise prefer columns closer to center
        let column_preference = [3, 2, 4, 1, 5, 0, 6];
        for &col in &column_preference {
            if self.is_valid_move(col) {
                self.make_move(col);
                return Some(col);
            }
        }
        
        None
    }
    
    // Count the total number of pieces on the board
    fn count_pieces(&self) -> usize {
        let mut count = 0;
        for row in 0..6 {
            for col in 0..7 {
                if self.board[row][col] != Connect4Cell::Empty {
                    count += 1;
                }
            }
        }
        count
    }
    
    // Find an immediate winning move for the specified player
    fn find_winning_move(&self, player: Connect4Cell) -> Option<usize> {
        for col in 0..7 {
            if !self.is_valid_move(col) {
                continue;
            }
            
            // Create a board copy and make the move
            let mut board_copy = self.clone();
            
            // Find the row where the piece will land
            let mut row = 5;
            while row > 0 && board_copy.board[row][col] != Connect4Cell::Empty {
                row -= 1;
            }
            
            // Place piece and check for win
            board_copy.board[row][col] = player;
            board_copy.last_move = Some((row, col));
            
            if board_copy.check_winner_minimax() {
                return Some(col);
            }
        }
        None
    }
    
    // Find double threat situations - where we can create two winning threats in one move
    // This is a forced win in 2 moves
    fn find_forced_win_in_two(&self) -> Option<usize> {
        for col in 0..7 {
            if !self.is_valid_move(col) {
                continue;
            }
            
            // Create a board copy and make the move
            let mut board_copy = self.clone();
            
            // Find the row where the piece will land
            let mut row = 5;
            while row > 0 && board_copy.board[row][col] != Connect4Cell::Empty {
                row -= 1;
            }
            
            // Place AI piece
            board_copy.board[row][col] = Connect4Cell::Yellow;
            
            // Count how many winning moves AI would have after this move
            let mut winning_moves = 0;
            for next_col in 0..7 {
                if !board_copy.is_valid_move(next_col) {
                    continue;
                }
                
                // Create another copy for checking the next move
                let mut next_board = board_copy.clone();
                
                // Find the row for the next move
                let mut next_row = 5;
                while next_row > 0 && next_board.board[next_row][next_col] != Connect4Cell::Empty {
                    next_row -= 1;
                }
                
                // Place AI piece for the second move
                next_board.board[next_row][next_col] = Connect4Cell::Yellow;
                next_board.last_move = Some((next_row, next_col));
                
                if next_board.check_winner_minimax() {
                    winning_moves += 1;
                }
            }
            
            // If we found at least two winning moves, this is a forced win
            if winning_moves >= 2 {
                return Some(col);
            }
        }
        None
    }
    
    // Find trap setups - positions that lead to future winning opportunities
    fn find_trap_setup(&self) -> Option<usize> {
        let mut best_col = None;
        let mut max_threats = 0;
        
        for col in 0..7 {
            if !self.is_valid_move(col) {
                continue;
            }
            
            // Create a board copy and make the move
            let mut board_copy = self.clone();
            
            // Find the row where the piece will land
            let mut row = 5;
            while row > 0 && board_copy.board[row][col] != Connect4Cell::Empty {
                row -= 1;
            }
            
            // Place AI piece
            board_copy.board[row][col] = Connect4Cell::Yellow;
            
            // Simulate opponent's possible responses and count our threats after each
            let mut min_opponent_threats = i32::MAX;
            let mut our_max_future_threats = 0;
            
            // Check each possible opponent response
            for opp_col in 0..7 {
                if !board_copy.is_valid_move(opp_col) {
                    continue;
                }
                
                // Create another board copy for opponent's move
                let mut opp_board = board_copy.clone();
                
                // Find row for opponent's move
                let mut opp_row = 5;
                while opp_row > 0 && opp_board.board[opp_row][opp_col] != Connect4Cell::Empty {
                    opp_row -= 1;
                }
                
                // Place opponent's piece
                opp_board.board[opp_row][opp_col] = Connect4Cell::Red;
                
                // Count threats for both players after this sequence
                let our_threats = opp_board.count_threats(Connect4Cell::Yellow);
                let opponent_threats = opp_board.count_threats(Connect4Cell::Red);
                
                // Check if opponent can win after their move
                if opponent_threats == 0 {
                    // Update our minimum threat count among all opponent responses
                    if our_threats > our_max_future_threats {
                        our_max_future_threats = our_threats;
                    }
                    
                    if opponent_threats < min_opponent_threats {
                        min_opponent_threats = opponent_threats;
                    }
                }
            }
            
            // If we found a position with more potential threats than current best,
            // and opponent has no immediate winning response, update best move
            if our_max_future_threats > max_threats && min_opponent_threats == 0 {
                max_threats = our_max_future_threats;
                best_col = Some(col);
            }
        }
        
        best_col
    }
    
    // Count how many potential winning threats a player has
    fn count_threats(&self, player: Connect4Cell) -> i32 {
        let mut threats = 0;
        
        // Check horizontal threats
        for row in 0..6 {
            for col in 0..4 {
                let window = [
                    self.board[row][col],
                    self.board[row][col+1],
                    self.board[row][col+2],
                    self.board[row][col+3]
                ];
                
                let player_count = window.iter().filter(|&&cell| cell == player).count();
                let empty_count = window.iter().filter(|&&cell| cell == Connect4Cell::Empty).count();
                
                // A threat is three pieces in a row with an empty space
                if player_count == 3 && empty_count == 1 {
                    threats += 1;
                }
            }
        }
        
        // Check vertical threats
        for col in 0..7 {
            for row in 0..3 {
                let window = [
                    self.board[row][col],
                    self.board[row+1][col],
                    self.board[row+2][col],
                    self.board[row+3][col]
                ];
                
                let player_count = window.iter().filter(|&&cell| cell == player).count();
                let empty_count = window.iter().filter(|&&cell| cell == Connect4Cell::Empty).count();
                
                if player_count == 3 && empty_count == 1 {
                    threats += 1;
                }
            }
        }
        
        // Check diagonal threats (positive slope)
        for row in 0..3 {
            for col in 0..4 {
                let window = [
                    self.board[row][col],
                    self.board[row+1][col+1],
                    self.board[row+2][col+2],
                    self.board[row+3][col+3]
                ];
                
                let player_count = window.iter().filter(|&&cell| cell == player).count();
                let empty_count = window.iter().filter(|&&cell| cell == Connect4Cell::Empty).count();
                
                if player_count == 3 && empty_count == 1 {
                    threats += 1;
                }
            }
        }
        
        // Check diagonal threats (negative slope)
        for row in 3..6 {
            for col in 0..4 {
                let window = [
                    self.board[row][col],
                    self.board[row-1][col+1],
                    self.board[row-2][col+2],
                    self.board[row-3][col+3]
                ];
                
                let player_count = window.iter().filter(|&&cell| cell == player).count();
                let empty_count = window.iter().filter(|&&cell| cell == Connect4Cell::Empty).count();
                
                if player_count == 3 && empty_count == 1 {
                    threats += 1;
                }
            }
        }
        
        threats
    }

    // Minimax algorithm with alpha-beta pruning
    fn minimax(&self, depth: i32, maximizing_player: bool, mut alpha: i32, mut beta: i32) -> (i32, i32) {
        // Terminal conditions: depth reached or game over
        if depth == 0 || self.is_board_full() || self.check_winner_minimax() {
            return (-1, self.evaluate_board());
        }
        
        // Check transposition table
        let board_hash = self.hash_board();
        if let Ok(table) = self.transposition_table.lock() {
            if let Some(&(score, stored_depth)) = table.get(&board_hash) {
                // Only use stored value if it was computed to at least the current depth
                if stored_depth >= depth {
                    return (-1, score);
                }
            }
        }
        
        // Column ordering for better alpha-beta pruning (start with center columns)
        let column_order = [3, 2, 4, 1, 5, 0, 6];
        let current_player = if maximizing_player { Connect4Cell::Yellow } else { Connect4Cell::Red };
        
        // For AI (maximizing) - higher score is better
        // For player (minimizing) - lower score is better
        if maximizing_player {
            let mut best_score = i32::MIN;
            let mut best_col = -1;
            
            // Try each column in order of preference
            for &col in &column_order {
                if self.is_valid_move(col) {
                    // Create a board copy and make the move
                    let mut board_copy = self.clone();
                    
                    // Find the row where the piece will land
                    let mut row = 5;
                    while row > 0 && board_copy.board[row][col] != Connect4Cell::Empty {
                        row -= 1;
                    }
                    
                    board_copy.board[row][col] = current_player;
                    board_copy.last_move = Some((row, col));
                    
                    // Check for immediate win to speed up search
                    if board_copy.check_winner_minimax() {
                        // Update transposition table using the mutex
                        if let Ok(mut table) = self.transposition_table.lock() {
                            table.insert(board_hash, (1000000, depth));
                        }
                        return (col as i32, 1000000);
                    }
                    
                    // Recursive call
                    let (_, score) = board_copy.minimax(depth - 1, false, alpha, beta);
                    
                    if score > best_score {
                        best_score = score;
                        best_col = col as i32;
                    }
                    
                    // Alpha-beta pruning
                    alpha = alpha.max(best_score);
                    if beta <= alpha {
                        break; // Beta cutoff
                    }
                }
            }
            
            // Store result in transposition table - but only at the root call
            if depth > 1 {
                // Safety check to avoid overflow: only insert if we have a valid best_col
                if best_col >= 0 {
                    // Update transposition table using the mutex
                    if let Ok(mut table) = self.transposition_table.lock() {
                        table.insert(board_hash, (best_score, depth));
                    }
                }
            }
            
            (best_col, best_score)
        } else {
            // Similar minimizing player code with table updates
            let mut best_score = i32::MAX;
            let mut best_col = -1;
            
            // Try each column in order of preference
            for &col in &column_order {
                if self.is_valid_move(col) {
                    // Create a board copy and make the move
                    let mut board_copy = self.clone();
                    
                    // Find the row where the piece will land
                    let mut row = 5;
                    while row > 0 && board_copy.board[row][col] != Connect4Cell::Empty {
                        row -= 1;
                    }
                    
                    board_copy.board[row][col] = current_player;
                    board_copy.last_move = Some((row, col));
                    
                    // Check for immediate win to speed up search
                    if board_copy.check_winner_minimax() {
                        // Update transposition table using the mutex
                        if let Ok(mut table) = self.transposition_table.lock() {
                            table.insert(board_hash, (-1000000, depth));
                        }
                        return (col as i32, -1000000);
                    }
                    
                    // Recursive call
                    let (_, score) = board_copy.minimax(depth - 1, true, alpha, beta);
                    
                    if score < best_score {
                        best_score = score;
                        best_col = col as i32;
                    }
                    
                    // Alpha-beta pruning
                    beta = beta.min(best_score);
                    if beta <= alpha {
                        break; // Alpha cutoff
                    }
                }
            }
            
            // Store result in transposition table - but only at important depths
            // to avoid table bloat and overflows
            if depth > 1 {
                // Safety check to avoid overflow: only insert if we have a valid best_col
                if best_col >= 0 {
                    // Update transposition table using the mutex
                    if let Ok(mut table) = self.transposition_table.lock() {
                        table.insert(board_hash, (best_score, depth));
                    }
                }
            }
            
            (best_col, best_score)
        }
    }
    
    // Evaluate the board state - higher score favors AI (Yellow)
    fn evaluate_board(&self) -> i32 {
        // Check if game is won
        if self.check_winner_minimax() {
            if let Some((row, col)) = self.last_move {
                if self.board[row][col] == Connect4Cell::Yellow {
                    return 1000000; // AI wins
                } else {
                    return -1000000; // Player wins
                }
            }
        }
        
        let mut score = 0;
        
        // Add score for center column control (center is strategically valuable)
        for row in 0..6 {
            if self.board[row][3] == Connect4Cell::Yellow {
                score += 6; // Increased value for center control
            } else if self.board[row][3] == Connect4Cell::Red {
                score -= 6;
            }
        }
        
        // Score adjacent center columns with slightly less weight
        for row in 0..6 {
            for col in [2, 4] {
                if self.board[row][col] == Connect4Cell::Yellow {
                    score += 4;
                } else if self.board[row][col] == Connect4Cell::Red {
                    score -= 4;
                }
            }
        }
        
        // Score horizontal windows
        for row in 0..6 {
            for col in 0..4 {
                score += self.evaluate_window([
                    self.board[row][col], 
                    self.board[row][col+1], 
                    self.board[row][col+2], 
                    self.board[row][col+3]
                ], row);
            }
        }
        
        // Score vertical windows
        for col in 0..7 {
            for row in 0..3 {
                score += self.evaluate_window([
                    self.board[row][col], 
                    self.board[row+1][col], 
                    self.board[row+2][col], 
                    self.board[row+3][col]
                ], row);
            }
        }
        
        // Score diagonal windows (positive slope)
        for row in 0..3 {
            for col in 0..4 {
                score += self.evaluate_window([
                    self.board[row][col], 
                    self.board[row+1][col+1], 
                    self.board[row+2][col+2], 
                    self.board[row+3][col+3]
                ], row);
            }
        }
        
        // Score diagonal windows (negative slope)
        for row in 3..6 {
            for col in 0..4 {
                score += self.evaluate_window([
                    self.board[row][col], 
                    self.board[row-1][col+1], 
                    self.board[row-2][col+2], 
                    self.board[row-3][col+3]
                ], row);
            }
        }
        
        // Add specialized pattern recognition for horizontal threats at bottom
        for col in 0..4 {
            let bottom_row = 5;
            let window = [
                self.board[bottom_row][col],
                self.board[bottom_row][col+1],
                self.board[bottom_row][col+2],
                self.board[bottom_row][col+3]
            ];
            
            // Special scoring for horizontal patterns on bottom row
            // This addresses the issue where the AI missed the user's horizontal win
            let _yellow_count = window.iter().filter(|&&c| c == Connect4Cell::Yellow).count();
            let red_count = window.iter().filter(|&&c| c == Connect4Cell::Red).count();
            let empty_count = window.iter().filter(|&&c| c == Connect4Cell::Empty).count();
            
            if red_count == 2 && empty_count == 2 {
                // Dangerous potential bottom row setup by player
                score -= 50;  // Much higher penalty
            } else if red_count == 3 && empty_count == 1 {
                // Critical bottom row threat
                score -= 500; // Extremely high penalty
            }
        }
        
        // Score threats (3-in-a-row with open ends)
        score += self.count_threats(Connect4Cell::Yellow) * 25;
        score -= self.count_threats(Connect4Cell::Red) * 30; // Weight opponent threats higher
        
        // Add special score for stair-step patterns that lead to forced wins
        score += self.detect_stairstep_patterns();
        
        score
    }
    
    // Evaluate a window of 4 cells
    fn evaluate_window(&self, window: [Connect4Cell; 4], row: usize) -> i32 {
        let mut score = 0;
        
        let yellow_count = window.iter().filter(|&&c| c == Connect4Cell::Yellow).count();
        let red_count = window.iter().filter(|&&c| c == Connect4Cell::Red).count();
        let empty_count = window.iter().filter(|&&c| c == Connect4Cell::Empty).count();
        
        // If both players have pieces in the window, it can't be completed by either
        if yellow_count > 0 && red_count > 0 {
            return 0;
        }
        
        // Score AI (Yellow) possibilities with exponential scoring
        if yellow_count > 0 && red_count == 0 {
            if yellow_count == 4 {
                score += 1000; // Should be caught by win check, but just in case
            } else if yellow_count == 3 && empty_count == 1 {
                score += 100; // Three in a row is very valuable
            } else if yellow_count == 2 && empty_count == 2 {
                score += 10;  // Two in a row with space
            } else if yellow_count == 1 && empty_count == 3 {
                score += 1;   // One piece
            }
        }
        
        // Score opponent (Red) possibilities with exponential scoring
        if red_count > 0 && yellow_count == 0 {
            if red_count == 4 {
                score -= 1000; // Should be caught by win check
            } else if red_count == 3 && empty_count == 1 {
                score -= 120; // Block opponent's three in a row aggressively
            } else if red_count == 2 && empty_count == 2 {
                score -= 12;  // Preemptively block two in a row
            } else if red_count == 1 && empty_count == 3 {
                score -= 1;   // One piece
            }
        }
        
        // Apply row-based multiplier to prioritize bottom rows
        // Bottom row is critical in Connect4
        if row == 5 {  // Bottom row
            score *= 3;  // Triple the importance
        } else if row == 4 {  // Second from bottom
            score *= 2;  // Double the importance
        }
        
        score
    }
    
    // Check for a winner specifically for minimax, without modifying game state
    fn check_winner_minimax(&self) -> bool {
        // No need to check if no moves have been made
        if self.last_move.is_none() {
            return false;
        }
        
        let (last_row, last_col) = self.last_move.unwrap();
        let piece = self.board[last_row][last_col];
        
        if piece == Connect4Cell::Empty {
            return false;
        }
        
        // Check horizontal
        let mut count = 0;
        for col in 0..7 {
            if self.board[last_row][col] == piece {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
        }
        
        // Check vertical
        count = 0;
        for row in 0..6 {
            if self.board[row][last_col] == piece {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
        }
        
        // Check diagonal (up-right)
        count = 0;
        let mut row = last_row as i32;
        let mut col = last_col as i32;
        
        // Move to bottom-left of the diagonal
        while row < 5 && col > 0 {
            row += 1;
            col -= 1;
        }
        
        // Check the whole diagonal
        while row >= 0 && col < 7 {
            if row >= 0 && row < 6 && col >= 0 && col < 7 &&
               self.board[row as usize][col as usize] == piece {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
            row -= 1;
            col += 1;
        }
        
        // Check diagonal (up-left)
        count = 0;
        row = last_row as i32;
        col = last_col as i32;
        
        // Move to bottom-right of the diagonal
        while row < 5 && col < 6 {
            row += 1;
            col += 1;
        }
        
        // Check the whole diagonal
        while row >= 0 && col >= 0 {
            if row >= 0 && row < 6 && col >= 0 && col < 7 &&
               self.board[row as usize][col as usize] == piece {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
            row -= 1;
            col -= 1;
        }
        
        false
    }
    
    fn check_winner(&self) -> bool {
        // No need to check if no moves have been made
        if self.last_move.is_none() {
            return false;
        }
        
        let (last_row, last_col) = self.last_move.unwrap();
        let piece = self.board[last_row][last_col];
        
        if piece == Connect4Cell::Empty {
            return false;
        }
        
        // Check horizontal
        let mut count = 0;
        for col in 0..7 {
            if self.board[last_row][col] == piece {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
        }
        
        // Check vertical
        count = 0;
        for row in 0..6 {
            if self.board[row][last_col] == piece {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
        }
        
        // Check diagonal (up-right)
        count = 0;
        let mut row = last_row as i32;
        let mut col = last_col as i32;
        
        // Move to bottom-left of the diagonal
        while row < 5 && col > 0 {
            row += 1;
            col -= 1;
        }
        
        // Check the whole diagonal
        while row >= 0 && col < 7 {
            if row >= 0 && row < 6 && col >= 0 && col < 7 &&
               self.board[row as usize][col as usize] == piece {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
            row -= 1;
            col += 1;
        }
        
        // Check diagonal (up-left)
        count = 0;
        row = last_row as i32;
        col = last_col as i32;
        
        // Move to bottom-right of the diagonal
        while row < 5 && col < 6 {
            row += 1;
            col += 1;
        }
        
        // Check the whole diagonal
        while row >= 0 && col >= 0 {
            if row >= 0 && row < 6 && col >= 0 && col < 7 &&
               self.board[row as usize][col as usize] == piece {
                count += 1;
                if count >= 4 {
                    return true;
                }
            } else {
                count = 0;
            }
            row -= 1;
            col -= 1;
        }
        
        false
    }
    
    fn is_board_full(&self) -> bool {
        // Check if the top row is full
        self.board[0].iter().all(|&cell| cell != Connect4Cell::Empty)
    }
    
    fn render_board(&self) -> String {
        let mut board_str = String::new();
        
        // Board rows
        for row in 0..6 {
            for col in 0..7 {
                board_str.push_str(&format!("{}", self.board[row][col]));
            }
            board_str.push('\n');
        }
        
        // Column numbers at the bottom
        board_str.push_str("1️⃣2️⃣3️⃣4️⃣5️⃣6️⃣7️⃣");
        
        board_str
    }
    
    fn render_status(&self, player1: &serenity::User, player2: Option<&serenity::User>) -> String {
        let red_player = format!("**{}** (🔴)", player1.name);
        let yellow_player = if let Some(p2) = player2 {
            format!("**{}** (🟡)", p2.name)
        } else {
            "The computer (🟡)".to_string()
        };
        
        let communist_win_messages = [
            "has seized control of the board! The revolution succeeds!",
            "has united the workers and achieved victory!",
            "has spread the revolution across the board!",
            "has established the dictatorship of the proletariat!",
            "has proven that the collective will always triumph!",
        ];
        
        let capitalist_messages = [
            "has temporarily gained control of the means of production...",
            "has shown reactionary tendencies, but the revolution will continue!",
            "has secured a victory, but the struggle continues!",
            "has won this battle, but not the class war!",
            "has succeeded in this game, but history is on our side!",
        ];
        
        let msg_idx = rand::random::<usize>() % communist_win_messages.len();
        
        if self.game_over {
            if let Some(winner) = self.winner {
                match winner {
                    Connect4Cell::Red => format!("{} {}", red_player, communist_win_messages[msg_idx]),
                    Connect4Cell::Yellow => {
                        if self.ai_mode {
                            format!("{} {}", yellow_player, capitalist_messages[msg_idx])
                        } else {
                            format!("{} {}", yellow_player, communist_win_messages[msg_idx])
                        }
                    },
                    _ => "Unknown winner".to_string()
                }
            } else {
                "The game ended in a draw! Perfect balance, as all things should be!".to_string()
            }
        } else {
            match self.current_player {
                Connect4Cell::Red => format!("It's {}'s turn", red_player),
                Connect4Cell::Yellow => format!("It's {}'s turn", yellow_player),
                _ => "Unknown player's turn".to_string()
            }
        }
    }

    // Fix the hash_board function to avoid overflow
    fn hash_board(&self) -> u64 {
        let mut hash: u64 = 0;
        
        // Use Zobrist hashing approach - XOR operations don't overflow
        for row in 0..6 {
            for col in 0..7 {
                let cell_value = match self.board[row][col] {
                    Connect4Cell::Empty => 0u64,
                    Connect4Cell::Red => 1u64,
                    Connect4Cell::Yellow => 2u64,
                };
                
                if cell_value > 0 {
                    // Simple hash that won't overflow - use position and value to create unique bits
                    // (row * 7 + col) gives unique position index (0-41)
                    // cell_value gives piece type (1-2)
                    // Shift by position to get unique bit patterns
                    let position_hash = ((row * 7 + col) as u64) << 1;
                    let piece_hash = cell_value;
                    let cell_hash = position_hash ^ piece_hash;
                    
                    // XOR doesn't overflow
                    hash ^= cell_hash;
                    
                    // Add a second mixing component based on position to improve hash distribution
                    hash ^= ((row * 13 + col * 17) as u64) * cell_value;
                }
            }
        }
        
        // Ensure non-zero hash (extremely unlikely but just in case)
        if hash == 0 {
            hash = 1;
        }
        
        hash
    }

    // Add new function to detect stair-step patterns
    fn detect_stairstep_patterns(&self) -> i32 {
        let mut score = 0;
        
        // Check for yellow (AI) stair patterns that lead to forced wins
        for col in 0..4 {
            for row in 2..6 {
                // Check pattern: yellow piece, empty above, yellow diagonally up
                if row >= 3 && col <= 5 &&
                   self.board[row][col] == Connect4Cell::Yellow &&
                   self.board[row-1][col] == Connect4Cell::Empty &&
                   self.board[row-1][col+1] == Connect4Cell::Yellow {
                    score += 25;
                }
                
                // Check another common forced win pattern
                if row >= 3 && col <= 4 &&
                   self.board[row][col] == Connect4Cell::Yellow &&
                   self.board[row-1][col+1] == Connect4Cell::Yellow &&
                   self.board[row-2][col+2] == Connect4Cell::Empty &&
                   self.board[row-1][col+2] == Connect4Cell::Empty {
                    score += 40;
                }
            }
        }
        
        // Check for and defend against red (player) stair patterns
        for col in 0..4 {
            for row in 2..6 {
                // Check pattern: red piece, empty above, red diagonally up
                if row >= 3 && col <= 5 &&
                   self.board[row][col] == Connect4Cell::Red &&
                   self.board[row-1][col] == Connect4Cell::Empty &&
                   self.board[row-1][col+1] == Connect4Cell::Red {
                    score -= 30;
                }
                
                // Check another common forced win pattern for opponent
                if row >= 3 && col <= 4 &&
                   self.board[row][col] == Connect4Cell::Red &&
                   self.board[row-1][col+1] == Connect4Cell::Red &&
                   self.board[row-2][col+2] == Connect4Cell::Empty &&
                   self.board[row-1][col+2] == Connect4Cell::Empty {
                    score -= 50;
                }
            }
        }
        
        score
    }

    // Add an opening book for the first few moves
    fn get_opening_move(&self) -> Option<usize> {
        let piece_count = self.count_pieces();
        
        // Only use opening book for the first few moves
        if piece_count > 4 {
            return None;
        }
        
        // First move as AI (piece_count = 1, player just made first move)
        if piece_count == 1 {
            // If player played in center, play adjacent to center
            if self.board[5][3] == Connect4Cell::Red {
                return Some(2); // Play column 2 (adjacent to center)
            }
            
            // If player played on sides, play center
            return Some(3); // Play center column
        }
        
        // Second move as AI (piece_count = 3, player made 2 moves, AI made 1)
        if piece_count == 3 {
            // If we control center, develop adjacent positions
            if self.board[5][3] == Connect4Cell::Yellow {
                // Check where player's pieces are
                if self.board[5][0] == Connect4Cell::Red || self.board[5][1] == Connect4Cell::Red {
                    return Some(2); // Play defensive on left side
                } else if self.board[5][5] == Connect4Cell::Red || self.board[5][6] == Connect4Cell::Red {
                    return Some(4); // Play defensive on right side
                }
                return Some(4); // Default to right-center development
            }
            
            // If we don't control center, try to create dual threats
            if self.board[5][2] == Connect4Cell::Yellow {
                if self.is_valid_move(4) {
                    return Some(4);
                }
            } else if self.board[5][4] == Connect4Cell::Yellow {
                if self.is_valid_move(2) {
                    return Some(2);
                }
            }
        }
        
        None
    }
}

// Comrade Clicker Game
struct ComradeClicker {
    grid_size: usize,
    active_cell: Option<usize>,
    round: usize,
    total_rounds: usize,
    start_time: Instant,
    click_times: Vec<Duration>,
    game_over: bool,
}

impl ComradeClicker {
    fn new(grid_size: usize, total_rounds: usize) -> Self {
        ComradeClicker {
            grid_size,
            active_cell: None,
            round: 0,
            total_rounds,
            start_time: Instant::now(),
            click_times: Vec::with_capacity(total_rounds),
            game_over: false,
        }
    }
    
    fn start_round(&mut self) {
        let total_cells = self.grid_size * self.grid_size;
        let mut rng = rand::thread_rng();
        self.active_cell = Some(rng.gen_range(0..total_cells));
        self.start_time = Instant::now();
    }
    
    fn handle_click(&mut self, position: usize) -> bool {
        if self.game_over || self.active_cell.is_none() {
            return false;
        }
        
        if Some(position) == self.active_cell {
            // Correct cell clicked
            let click_time = self.start_time.elapsed();
            self.click_times.push(click_time);
            self.round += 1;
            
            if self.round >= self.total_rounds {
                self.game_over = true;
                self.active_cell = None;
            } else {
                self.start_round();
            }
            
            return true;
        }
        
        // Wrong cell clicked
        false
    }
    
    fn get_average_time(&self) -> Option<Duration> {
        if self.click_times.is_empty() {
            return None;
        }
        
        let total = self.click_times.iter().sum::<Duration>();
        Some(total / self.click_times.len() as u32)
    }
    
    fn get_best_time(&self) -> Option<Duration> {
        self.click_times.iter().min().copied()
    }
    
    fn get_score(&self) -> Option<f64> {
        // Score is the average reaction time in milliseconds
        self.get_average_time().map(|duration| {
            duration.as_secs_f64() * 1000.0
        })
    }
    
    fn render_status(&self) -> String {
        if self.game_over {
            let avg_time = self.get_average_time()
                .map_or_else(|| "N/A".to_string(), |t| format!("{:.2}ms", t.as_secs_f64() * 1000.0));
            
            let best_time = self.get_best_time()
                .map_or_else(|| "N/A".to_string(), |t| format!("{:.2}ms", t.as_secs_f64() * 1000.0));
            
            format!(
                "**Game Complete!**\n\
                Rounds: {}/{}\n\
                Average time: {}\n\
                Best time: {}\n\
                The Motherland thanks you for your swift labor, comrade!",
                self.round, self.total_rounds, avg_time, best_time
            )
        } else {
            if self.active_cell.is_none() {
                format!(
                    "**Preparing Round {}**\n\
                    Get ready to click the red square when it appears!",
                    self.round + 1
                )
            } else {
                format!(
                    "**Round {}/{}**\n\
                    Click the red square as quickly as possible!",
                    self.round + 1, self.total_rounds
                )
            }
        }
    }
}

/// Shows available games
#[poise::command(slash_command, prefix_command)]
pub async fn game(
    ctx: crate::Context<'_>,
) -> Result<(), CommandError> {
    ctx.say(
        "**☭ Available Communist Games ☭**\n\n\
        Use one of the following commands to play:\n\
        • `/tictactoe [@user]` - Play Tic-Tac-Toe against another user or the computer\n\
        • `/clicker` - Test your reaction time in Comrade Clicker\n\
        • `/connect4 [@user]` - Play Connect 4: People's Revolution Edition\n\n\
        Glory to the collective!"
    ).await?;
    
    Ok(())
}

/// Play a game of tic-tac-toe alone or with a friend
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn tictactoe(
    ctx: crate::Context<'_>,
    #[description = "The user to play against (leave empty to play against the computer)"] 
    opponent: Option<serenity::User>,
) -> Result<(), CommandError> {
    // Check if the opponent is the same as the player
    if let Some(ref user) = opponent {
        if user.id == ctx.author().id {
            ctx.say("You can't play against yourself! Choose another player or leave empty to play against the computer.").await?;
            return Ok(());
        }
        
        if user.bot {
            ctx.say("You can't play against a bot! Choose a human player or leave empty to play against the computer.").await?;
            return Ok(());
        }
    }
    
    let author = ctx.author().clone();
    let mut game = TicTacToe::new(author.id, opponent.as_ref().map(|u| u.id));
    
    // Create initial message
    let msg = ctx.send(|m| {
        m.content(format!(
            "**☭ Communist Tic-Tac-Toe Game ☭**\n\n{}\n\n{}",
            game.render_board(),
            game.render_status(&author, opponent.as_ref())
        ))
        .components(|c| {
            c.create_action_row(|row| {
                for i in 0..3 {
                    row.create_button(|b| {
                        b.custom_id(format!("button_{}", i))
                        .label(format!("{}", i + 1))
                        .style(serenity::ButtonStyle::Secondary)
                    });
                }
                row
            })
            .create_action_row(|row| {
                for i in 3..6 {
                    row.create_button(|b| {
                        b.custom_id(format!("button_{}", i))
                        .label(format!("{}", i + 1))
                        .style(serenity::ButtonStyle::Secondary)
                    });
                }
                row
            })
            .create_action_row(|row| {
                for i in 6..9 {
                    row.create_button(|b| {
                        b.custom_id(format!("button_{}", i))
                        .label(format!("{}", i + 1))
                        .style(serenity::ButtonStyle::Secondary)
                    });
                }
                row
            })
        })
    }).await?;
    
    // Computer moves first if it's an AI game and computer starts as X
    if game.ai_mode && game.current_player == Cell::O {
        // Wait a moment to make it look like the computer is thinking
        tokio::time::sleep(Duration::from_millis(1500)).await;
        
        if let Some(ai_pos) = game.ai_move() {
            // Update message with the computer's move
            msg.edit(ctx, |m| {
                m.content(format!(
                    "**☭ Communist Tic-Tac-Toe Game ☭**\n\n{}\n\nComputer played position {}.\n{}",
                    game.render_board(), 
                    ai_pos + 1,
                    game.render_status(&author, opponent.as_ref())
                ))
                .components(|c| {
                    if game.game_over {
                        c // No components if game is over
                    } else {
                        for row in 0..3 {
                            c.create_action_row(|r| {
                                for col in 0..3 {
                                    let i = row * 3 + col;
                                    r.create_button(|b| {
                                        b.custom_id(format!("button_{}", i))
                                        .label(format!("{}", i + 1))
                                        .style(serenity::ButtonStyle::Secondary)
                                        .disabled(game.board[i] != Cell::Empty)
                                    });
                                }
                                r
                            });
                        }
                        c
                    }
                })
            }).await?;
        }
    }
    
    // Handle button interactions
    while !game.game_over {
        // Wait for someone to click a button
        if let Some(press) = serenity::CollectComponentInteraction::new(ctx)
            .filter(move |press| {
                // Only allow the current player to make a move
                let user_id = press.user.id;
                
                if game.current_player == Cell::X {
                    user_id == author.id // Only author can play as X
                } else {
                    // O's turn
                    if game.ai_mode {
                        false // Computer's turn, no user interaction allowed
                    } else {
                        // Check if the interaction is from the opponent
                        if let Some(opponent_id) = game.opponent {
                            user_id == opponent_id
                        } else {
                            false
                        }
                    }
                }
            })
            .timeout(Duration::from_secs(300)) // 5 minutes timeout
            .await
        {
            // Acknowledge the button press
            press.defer(ctx).await?;
            
            // Extract position from the button custom_id (format: "button_X")
            let position = press.data.custom_id
                .strip_prefix("button_")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
                
            // Make the move
            if game.make_move(position) {
                // Update message
                msg.edit(ctx, |m| {
                    m.content(format!(
                        "**☭ Communist Tic-Tac-Toe Game ☭**\n\n{}\n\n{}",
                        game.render_board(),
                        game.render_status(&author, opponent.as_ref())
                    ))
                    .components(|c| {
                        if game.game_over {
                            c // No components if game is over
                        } else {
                            for row in 0..3 {
                                c.create_action_row(|r| {
                                    for col in 0..3 {
                                        let i = row * 3 + col;
                                        r.create_button(|b| {
                                            b.custom_id(format!("button_{}", i))
                                            .label(format!("{}", i + 1))
                                            .style(serenity::ButtonStyle::Secondary)
                                            .disabled(game.board[i] != Cell::Empty)
                                        });
                                    }
                                    r
                                });
                            }
                            c
                        }
                    })
                }).await?;
                
                // If game is against computer and it's the computer's turn
                if game.ai_mode && !game.game_over && game.current_player == Cell::O {
                    // Wait a moment to make it look like the computer is thinking
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    
                    if let Some(ai_pos) = game.ai_move() {
                        // Update message with the computer's move
                        msg.edit(ctx, |m| {
                            m.content(format!(
                                "**☭ Communist Tic-Tac-Toe Game ☭**\n\n{}\n\nComputer played position {}.\n{}",
                                game.render_board(), 
                                ai_pos + 1,
                                game.render_status(&author, opponent.as_ref())
                            ))
                            .components(|c| {
                                if game.game_over {
                                    c // No components if game is over
                                } else {
                                    for row in 0..3 {
                                        c.create_action_row(|r| {
                                            for col in 0..3 {
                                                let i = row * 3 + col;
                                                r.create_button(|b| {
                                                    b.custom_id(format!("button_{}", i))
                                                    .label(format!("{}", i + 1))
                                                    .style(serenity::ButtonStyle::Secondary)
                                                    .disabled(game.board[i] != Cell::Empty)
                                                });
                                            }
                                            r
                                        });
                                    }
                                    c
                                }
                            })
                        }).await?;
                    }
                }
            }
        } else {
            // Timeout or error - end the game
            msg.edit(ctx, |m| {
                m.content(format!(
                    "**☭ Communist Tic-Tac-Toe Game ☭**\n\n{}\n\nGame abandoned due to inactivity!",
                    game.render_board()
                ))
                .components(|c| c)
            }).await?;
            break;
        }
    }
    
    // Game is over, final message
    if game.game_over {
        let final_message = if let Some(winner) = game.winner {
            match winner {
                Cell::X => "The glory of the X workers prevails!",
                Cell::O => "The triumph of the O collective is complete!",
                _ => "The game ended without a clear outcome."
            }
        } else {
            "A fair draw - the means of production have been equally distributed!"
        };
        
        msg.edit(ctx, |m| {
            m.content(format!(
                "**☭ Communist Tic-Tac-Toe Game ☭**\n\n{}\n\n{}\n\n{}", 
                game.render_board(),
                game.render_status(&author, opponent.as_ref()),
                final_message
            ))
            .components(|c| c) // Clear components
        }).await?;
    }
    
    Ok(())
}

/// Play Comrade Clicker - test your reaction time for the Motherland!
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn clicker(
    ctx: crate::Context<'_>,
) -> Result<(), CommandError> {
    let author = ctx.author().clone();
    let server_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => "DM".to_string(),
    };
    
    // Create a new clicker game (5x5 grid, 5 rounds)
    let grid_size = 5;
    let total_rounds = 5;
    let mut game = ComradeClicker::new(grid_size, total_rounds);
    
    // Create initial message
    let msg = ctx.send(|m| {
        m.content(format!(
            "**☭ Comrade Clicker ☭**\n\n\
            Welcome, Comrade {}! Your labor efficiency will be tested.\n\n\
            Rules:\n\
            • Click on the red square (🟥) as quickly as possible\n\
            • Complete {} rounds to get your score\n\
            • Your score is your average reaction time (lower is better)\n\n\
            {}",
            author.name, total_rounds, game.render_status()
        ))
        .components(|c| {
            // Add a start button
            c.create_action_row(|row| {
                row.create_button(|b| {
                    b.custom_id("start_game")
                    .label("Start Game")
                    .style(serenity::ButtonStyle::Success)
                    .emoji('✅')
                })
            })
        })
    }).await?;
    
    // Wait for the start button to be pressed
    if let Some(press) = serenity::CollectComponentInteraction::new(ctx)
        .filter(move |press| press.user.id == author.id)
        .timeout(Duration::from_secs(60))
        .await
    {
        press.defer(ctx).await?;
        
        // Start the first round
        game.start_round();
        
        // Main game loop
        while !game.game_over {
            // Update the message with the current board
            let active_cell = game.active_cell;
            
            msg.edit(ctx, |m| {
                m.content(format!(
                    "**☭ Comrade Clicker ☭**\n\n{}",
                    game.render_status()
                ))
                .components(|c| {
                    // Create grid of buttons
                    for row in 0..grid_size {
                        c.create_action_row(|action_row| {
                            for col in 0..grid_size {
                                let position = row * grid_size + col;
                                let is_active = Some(position) == active_cell;
                                
                                action_row.create_button(|b| {
                                    b.custom_id(format!("cell_{}", position))
                                    .label(" ")
                                    .style(if is_active {
                                        serenity::ButtonStyle::Danger
                                    } else {
                                        serenity::ButtonStyle::Secondary
                                    })
                                    .emoji(if is_active { '🔴' } else { '⬛' })
                                });
                            }
                            action_row
                        });
                    }
                    c
                })
            }).await?;
            
            // Wait for a button press
            if let Some(press) = serenity::CollectComponentInteraction::new(ctx)
                .filter(move |press| press.user.id == author.id)
                .timeout(Duration::from_secs(10))
                .await
            {
                press.defer(ctx).await?;
                
                // Extract position from the button custom_id (format: "cell_X")
                let position = press.data.custom_id
                    .strip_prefix("cell_")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                
                // Handle the click
                let correct = game.handle_click(position);
                
                if !correct {
                    // User clicked the wrong cell - flash a message
                    msg.edit(ctx, |m| {
                        m.content(format!(
                            "**☭ Comrade Clicker ☭**\n\n\
                            That was the wrong cell, comrade! Pay attention!\n\n\
                            {}",
                            game.render_status()
                        ))
                    }).await?;
                    
                    // Pause briefly to show error
                    tokio::time::sleep(Duration::from_millis(800)).await;
                }
            } else {
                // Timeout - end the game
                game.game_over = true;
                
                msg.edit(ctx, |m| {
                    m.content(format!(
                        "**☭ Comrade Clicker ☭**\n\n\
                        You were too slow, comrade! The Party is disappointed in your lack of commitment.\n\n\
                        Game abandoned after {} of {} rounds.",
                        game.round, game.total_rounds
                    ))
                    .components(|c| c)
                }).await?;
                
                return Ok(());
            }
        }
        
        // Game is over, show final score
        let score = game.get_score();
        
        // Save score to database if available
        if let Some(score_ms) = score {
            let db = &ctx.data().db;
            db.save_game_score(
                &author.id.to_string(),
                &server_id,
                &author.name,
                "clicker",
                score_ms,
            ).await?;
            
            // Get the user's best score
            let best_score = db.get_user_best_score(&author.id.to_string(), &server_id, "clicker").await?;
            
            // Get server leaderboard (top 3)
            let leaderboard = db.get_server_leaderboard(&server_id, "clicker", 3).await?;
            
            // Prepare leaderboard text
            let leaderboard_text = if leaderboard.is_empty() {
                "No other scores recorded yet".to_string()
            } else {
                let mut text = "**Top Reaction Times:**\n".to_string();
                for (i, (username, score)) in leaderboard.iter().enumerate() {
                    text.push_str(&format!("{}. **{}**: {:.2}ms\n", i+1, username, score));
                }
                text
            };
            
            let best_score_text = if let Some(best) = best_score {
                if (score_ms - best).abs() < 0.001 {
                    "This is a new personal best!".to_string()
                } else {
                    format!("Your best score: {:.2}ms", best)
                }
            } else {
                "This is your first score!".to_string()
            };
            
            msg.edit(ctx, |m| {
                m.content(format!(
                    "**☭ Comrade Clicker ☭**\n\n\
                    **Game Complete!**\n\
                    Comrade {}, your average reaction time was: **{:.2}ms**\n\
                    {}\n\n\
                    {}\n\n\
                    The Motherland thanks you for your swift labor!",
                    author.name, score_ms, best_score_text, leaderboard_text
                ))
                .components(|c| {
                    c.create_action_row(|row| {
                        row.create_button(|b| {
                            b.custom_id("play_again")
                            .label("Play Again")
                            .style(serenity::ButtonStyle::Primary)
                        })
                    })
                })
            }).await?;
            
            // Wait for "Play Again" button press
            if let Some(press) = serenity::CollectComponentInteraction::new(ctx)
                .filter(move |press| {
                    press.user.id == author.id && press.data.custom_id == "play_again"
                })
                .timeout(Duration::from_secs(60))
                .await
            {
                press.defer(ctx).await?;
                
                // Instead of recursive call, just create a new message
                ctx.say("Starting a new Comrade Clicker game...").await?;
                
                // Start a new game directly using current context
                let mut new_game = ComradeClicker::new(grid_size, total_rounds);
                new_game.start_round();
                
                // Create a new message for the new game
                let new_msg = ctx.send(|m| {
                    m.content(format!(
                        "**☭ Comrade Clicker ☭**\n\n\
                        Welcome, Comrade {}! Your labor efficiency will be tested.\n\n\
                        Rules:\n\
                        • Click on the red square (🟥) as quickly as possible\n\
                        • Complete {} rounds to get your score\n\
                        • Your score is your average reaction time (lower is better)\n\n\
                        {}",
                        author.name, total_rounds, new_game.render_status()
                    ))
                    .components(|c| {
                        // Create grid of buttons for the new game
                        for row in 0..grid_size {
                            c.create_action_row(|action_row| {
                                for col in 0..grid_size {
                                    let position = row * grid_size + col;
                                    let is_active = Some(position) == new_game.active_cell;
                                    
                                    action_row.create_button(|b| {
                                        b.custom_id(format!("cell_{}", position))
                                        .label(" ")
                                        .style(if is_active {
                                            serenity::ButtonStyle::Danger
                                        } else {
                                            serenity::ButtonStyle::Secondary
                                        })
                                        .emoji(if is_active { '🔴' } else { '⬛' })
                                    });
                                }
                                action_row
                            });
                        }
                        c
                    })
                }).await?;
                
                // Start game loop for the new game
                let mut game_loop = true;
                while game_loop && !new_game.game_over {
                    if let Some(press) = serenity::CollectComponentInteraction::new(ctx)
                        .filter(move |press| press.user.id == author.id)
                        .timeout(Duration::from_secs(10))
                        .await
                    {
                        press.defer(ctx).await?;
                        
                        let position = press.data.custom_id
                            .strip_prefix("cell_")
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0);
                        
                        let correct = new_game.handle_click(position);
                        
                        if !correct {
                            new_msg.edit(ctx, |m| {
                                m.content(format!(
                                    "**☭ Comrade Clicker ☭**\n\n\
                                    That was the wrong cell, comrade! Pay attention!\n\n\
                                    {}",
                                    new_game.render_status()
                                ))
                            }).await?;
                            
                            tokio::time::sleep(Duration::from_millis(800)).await;
                        }
                        
                        // Update board
                        let active_cell = new_game.active_cell;
                        
                        new_msg.edit(ctx, |m| {
                            m.content(format!(
                                "**☭ Comrade Clicker ☭**\n\n{}",
                                new_game.render_status()
                            ))
                            .components(|c| {
                                if new_game.game_over {
                                    c
                                } else {
                                    for row in 0..grid_size {
                                        c.create_action_row(|action_row| {
                                            for col in 0..grid_size {
                                                let position = row * grid_size + col;
                                                let is_active = Some(position) == active_cell;
                                                
                                                action_row.create_button(|b| {
                                                    b.custom_id(format!("cell_{}", position))
                                                    .label(" ")
                                                    .style(if is_active {
                                                        serenity::ButtonStyle::Danger
                                                    } else {
                                                        serenity::ButtonStyle::Secondary
                                                    })
                                                    .emoji(if is_active { '🔴' } else { '⬛' })
                                                });
                                            }
                                            action_row
                                        });
                                    }
                                    c
                                }
                            })
                        }).await?;
                    } else {
                        // Timeout - end the game
                        new_game.game_over = true;
                        
                        new_msg.edit(ctx, |m| {
                            m.content(format!(
                                "**☭ Comrade Clicker ☭**\n\n\
                                You were too slow, comrade! The Party is disappointed in your lack of commitment.\n\n\
                                Game abandoned after {} of {} rounds.",
                                new_game.round, new_game.total_rounds
                            ))
                            .components(|c| c)
                        }).await?;
                        
                        game_loop = false;
                    }
                }
                
                // Display final score
                if new_game.game_over {
                    if let Some(score_ms) = new_game.get_score() {
                        let db = &ctx.data().db;
                        db.save_game_score(
                            &author.id.to_string(),
                            &server_id,
                            &author.name,
                            "clicker",
                            score_ms,
                        ).await?;
                        
                        let best_score = db.get_user_best_score(&author.id.to_string(), &server_id, "clicker").await?;
                        let leaderboard = db.get_server_leaderboard(&server_id, "clicker", 3).await?;
                        
                        let leaderboard_text = if leaderboard.is_empty() {
                            "No other scores recorded yet".to_string()
                        } else {
                            let mut text = "**Top Reaction Times:**\n".to_string();
                            for (i, (username, score)) in leaderboard.iter().enumerate() {
                                text.push_str(&format!("{}. **{}**: {:.2}ms\n", i+1, username, score));
                            }
                            text
                        };
                        
                        let best_score_text = if let Some(best) = best_score {
                            if (score_ms - best).abs() < 0.001 {
                                "This is a new personal best!".to_string()
                            } else {
                                format!("Your best score: {:.2}ms", best)
                            }
                        } else {
                            "This is your first score!".to_string()
                        };
                        
                        new_msg.edit(ctx, |m| {
                            m.content(format!(
                                "**☭ Comrade Clicker ☭**\n\n\
                                **Game Complete!**\n\
                                Comrade {}, your average reaction time was: **{:.2}ms**\n\
                                {}\n\n\
                                {}\n\n\
                                The Motherland thanks you for your swift labor!",
                                author.name, score_ms, best_score_text, leaderboard_text
                            ))
                            .components(|c| {
                                c.create_action_row(|row| {
                                    row.create_button(|b| {
                                        b.custom_id("play_again")
                                        .label("Play Again")
                                        .style(serenity::ButtonStyle::Primary)
                                    })
                                })
                            })
                        }).await?;
                    }
                }
                
                // Return early to avoid executing the rest of the function
                return Ok(());
            }
        } else {
            // User didn't press start button
            msg.edit(ctx, |m| {
                m.content(format!(
                    "**☭ Comrade Clicker ☭**\n\n\
                    Game abandoned. The Party notes your lack of enthusiasm, comrade."
                ))
                .components(|c| c)
            }).await?;
        }
    } else {
        // User didn't press start button
        msg.edit(ctx, |m| {
            m.content(format!(
                "**☭ Comrade Clicker ☭**\n\n\
                Game abandoned. The Party notes your lack of enthusiasm, comrade."
            ))
            .components(|c| c)
        }).await?;
    }
    
    Ok(())
}

/// Play a game of Connect 4 - The People's Revolution Edition!
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn connect4(
    ctx: crate::Context<'_>,
    #[description = "The user to play against (leave empty to play against the computer)"] 
    opponent: Option<serenity::User>,
) -> Result<(), CommandError> {
    // Check if the opponent is the same as the player
    if let Some(ref user) = opponent {
        if user.id == ctx.author().id {
            ctx.say("You can't play against yourself, comrade! Choose another player or leave empty to play against the capitalist AI.").await?;
            return Ok(());
        }
        
        if user.bot {
            ctx.say("You can't play against a bot! Choose a human player or leave empty to play against the computer.").await?;
            return Ok(());
        }
    }
    
    let author = ctx.author().clone();
    let mut game = Connect4::new(author.id, opponent.as_ref().map(|u| u.id));
    
    // Create initial message
    let msg = ctx.send(|m| {
        m.content(format!(
            "**☭ Connect 4: People's Revolution Edition ☭**\n\n{}\n\n{}",
            game.render_board(),
            game.render_status(&author, opponent.as_ref())
        ))
        .components(|c| {
            // Create 7 buttons for the 7 columns, split into two rows
            c.create_action_row(|row| {
                for i in 0..4 {
                    row.create_button(|b| {
                        b.custom_id(format!("col_{}", i))
                        .label(format!("{}", i + 1))
                        .style(serenity::ButtonStyle::Secondary)
                        .disabled(false)
                    });
                }
                row
            })
            .create_action_row(|row| {
                for i in 4..7 {
                    row.create_button(|b| {
                        b.custom_id(format!("col_{}", i))
                        .label(format!("{}", i + 1))
                        .style(serenity::ButtonStyle::Secondary)
                        .disabled(false)
                    });
                }
                row
            })
        })
    }).await?;
    
    // Computer moves first if it's an AI game and computer starts as Yellow (which it shouldn't since Red goes first)
    if game.ai_mode && game.current_player == Connect4Cell::Yellow {
        // Wait a moment to make it look like the computer is thinking
        tokio::time::sleep(Duration::from_millis(1500)).await;
        
        if let Some(ai_col) = game.ai_move() {
            // Update message with the computer's move
            msg.edit(ctx, |m| {
                m.content(format!(
                    "**☭ Connect 4: People's Revolution Edition ☭**\n\n{}\n\nComputer played in column {}.\n{}",
                    game.render_board(), 
                    ai_col + 1,
                    game.render_status(&author, opponent.as_ref())
                ))
                .components(|c| {
                    if game.game_over {
                        c // No components if game is over
                    } else {
                        c.create_action_row(|row| {
                            for i in 0..4 {
                                row.create_button(|b| {
                                    b.custom_id(format!("col_{}", i))
                                    .label(format!("{}", i + 1))
                                    .style(serenity::ButtonStyle::Secondary)
                                    .disabled(!game.is_valid_move(i))
                                });
                            }
                            row
                        })
                        .create_action_row(|row| {
                            for i in 4..7 {
                                row.create_button(|b| {
                                    b.custom_id(format!("col_{}", i))
                                    .label(format!("{}", i + 1))
                                    .style(serenity::ButtonStyle::Secondary)
                                    .disabled(!game.is_valid_move(i))
                                });
                            }
                            row
                        })
                    }
                })
            }).await?;
        }
    }
    
    // Handle button interactions
    while !game.game_over {
        // Wait for someone to click a button
        if let Some(press) = serenity::CollectComponentInteraction::new(ctx)
            .filter(move |press| {
                // Only allow the current player to make a move
                let user_id = press.user.id;
                
                if game.current_player == Connect4Cell::Red {
                    user_id == author.id // Only author can play as Red
                } else {
                    // Yellow's turn
                    if game.ai_mode {
                        false // Computer's turn, no user interaction allowed
                    } else {
                        // Check if the interaction is from the opponent
                        if let Some(opponent_id) = game.opponent {
                            user_id == opponent_id
                        } else {
                            false
                        }
                    }
                }
            })
            .timeout(Duration::from_secs(300)) // 5 minutes timeout
            .await
        {
            // Acknowledge the button press
            press.defer(ctx).await?;
            
            // Extract column from the button custom_id (format: "col_X")
            let column = press.data.custom_id
                .strip_prefix("col_")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
                
            // Make the move
            if game.make_move(column) {
                // Update message
                msg.edit(ctx, |m| {
                    m.content(format!(
                        "**☭ Connect 4: People's Revolution Edition ☭**\n\n{}\n\n{}",
                        game.render_board(),
                        game.render_status(&author, opponent.as_ref())
                    ))
                    .components(|c| {
                        if game.game_over {
                            c // No components if game is over
                        } else {
                            c.create_action_row(|row| {
                                for i in 0..4 {
                                    row.create_button(|b| {
                                        b.custom_id(format!("col_{}", i))
                                        .label(format!("{}", i + 1))
                                        .style(serenity::ButtonStyle::Secondary)
                                        .disabled(!game.is_valid_move(i))
                                    });
                                }
                                row
                            })
                            .create_action_row(|row| {
                                for i in 4..7 {
                                    row.create_button(|b| {
                                        b.custom_id(format!("col_{}", i))
                                        .label(format!("{}", i + 1))
                                        .style(serenity::ButtonStyle::Secondary)
                                        .disabled(!game.is_valid_move(i))
                                    });
                                }
                                row
                            })
                        }
                    })
                }).await?;
                
                // If game is against computer and it's the computer's turn
                if game.ai_mode && !game.game_over && game.current_player == Connect4Cell::Yellow {
                    // Wait a moment to make it look like the computer is thinking
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    
                    if let Some(ai_col) = game.ai_move() {
                        // Update message with the computer's move
                        msg.edit(ctx, |m| {
                            m.content(format!(
                                "**☭ Connect 4: People's Revolution Edition ☭**\n\n{}\n\nComputer played in column {}.\n{}",
                                game.render_board(), 
                                ai_col + 1,
                                game.render_status(&author, opponent.as_ref())
                            ))
                            .components(|c| {
                                if game.game_over {
                                    c // No components if game is over
                                } else {
                                    c.create_action_row(|row| {
                                        for i in 0..4 {
                                            row.create_button(|b| {
                                                b.custom_id(format!("col_{}", i))
                                                .label(format!("{}", i + 1))
                                                .style(serenity::ButtonStyle::Secondary)
                                                .disabled(!game.is_valid_move(i))
                                            });
                                        }
                                        row
                                    });
                                    c.create_action_row(|row| {
                                        for i in 4..7 {
                                            row.create_button(|b| {
                                                b.custom_id(format!("col_{}", i))
                                                .label(format!("{}", i + 1))
                                                .style(serenity::ButtonStyle::Secondary)
                                                .disabled(!game.is_valid_move(i))
                                            });
                                        }
                                        row
                                    })
                                }
                            })
                        }).await?;
                    }
                }
            }
        } else {
            // Timeout or error - end the game
            msg.edit(ctx, |m| {
                m.content(format!(
                    "**☭ Connect 4: People's Revolution Edition ☭**\n\n{}\n\nGame abandoned due to inactivity! The people demand action!",
                    game.render_board()
                ))
                .components(|c| c)
            }).await?;
            break;
        }
    }
    
    // Game is over, final message
    if game.game_over {
        let final_message = if let Some(winner) = game.winner {
            match winner {
                Connect4Cell::Red => "The Red Revolution has prevailed! The workers control the means of production!",
                Connect4Cell::Yellow => {
                    if game.ai_mode {
                        "The capitalist AI has temporarily gained control... but history is on our side!"
                    } else {
                        "The Yellow Faction has seized victory! All hail our new revolutionary leader!"
                    }
                },
                _ => "The game ended without a clear outcome."
            }
        } else {
            "A perfect draw - cooperation and equality have prevailed!"
        };
        
        msg.edit(ctx, |m| {
            m.content(format!(
                "**☭ Connect 4: People's Revolution Edition ☭**\n\n{}\n\n{}\n\n{}", 
                game.render_board(),
                game.render_status(&author, opponent.as_ref()),
                final_message
            ))
            .components(|c| c) // Clear components
        }).await?;
    }
    
    Ok(())
} 