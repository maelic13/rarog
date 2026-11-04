//! `position`: the board a `startpos` or `fen` line and its moves describe.

use crate::board::{Board, Move};

/// The position `args` describe, `None` when they name none (no arguments, or
/// a first word that is neither `startpos` nor `fen`), or why they cannot be
/// read.
pub fn parse(args: &[String]) -> Result<Option<Board>, String> {
    if args.is_empty() {
        return Ok(None);
    }

    let mut board = if args[0] == "startpos" {
        Board::default()
    } else if args[0] == "fen" {
        let fen_parts: Vec<&str> = args[1..]
            .iter()
            .take_while(|part| part.as_str() != "moves")
            .map(String::as_str)
            .collect();
        let fen = fen_parts.join(" ");
        match Board::from_fen(&fen) {
            Ok(board) => board,
            Err(_) => {
                return Err(String::from("Invalid FEN."));
            }
        }
    } else {
        return Ok(None);
    };

    let moves_start_index = args
        .iter()
        .position(|part| part == "moves")
        .map_or(args.len(), |index| index + 1);

    for move_text in &args[moves_start_index..] {
        if Move::from_uci(move_text).is_none() {
            return Err(format!("Illegal move: {move_text}"));
        }
        if !board.play_uci(move_text) {
            return Err(format!("Illegal move: {move_text}"));
        }
    }

    Ok(Some(board))
}
