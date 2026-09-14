//! Win At Chess (WAC) tactical suite — a `bench`-style diagnostic, not a gate.
//!
//! 300 classic tactical positions (`wac.epd`, standard public EPD with `bm`
//! best moves in SAN). The `wac [depth]` engine command searches each position
//! to a fixed depth and reports how many found an accepted best move, plus the
//! ids of the failures. The solved count is a *tactical-regression telltale*
//! for search-selectivity work (Phase 8.2 removes categorical check/PV
//! protections; a sudden drop here localizes a tactical regression long before
//! an SPRT can) — it is NOT a strength metric and never gates a change by
//! itself (SPRT remains the only verdict).
//!
//! Like `bench`, runs are deterministic: state is reset per position, so the
//! solved set is reproducible at a given depth and safe to compare across
//! candidates. A `tests/wac.rs` floor test guards against gross tactical
//! breakage in CI at a shallow depth.

use crate::board::moves::{CASTLE_KINGSIDE, CASTLE_QUEENSIDE, PROMO_KNIGHT};
use crate::board::{Board, Move, Piece, Square};

pub(crate) const DEFAULT_WAC_DEPTH: u16 = 10;

/// The raw EPD suite: `<placement> <stm> <castling> <ep> bm <san...>; id "WAC.n";`
const WAC_EPD: &str = include_str!("wac.epd");

/// One suite entry: a position, its accepted best moves (SAN), and its id.
pub struct WacPosition {
    pub fen: String,
    pub best_moves: Vec<String>,
    pub id: String,
}

/// Parses the embedded EPD into positions. EPD carries no move counters, so
/// `0 1` is appended to form a FEN. Malformed lines are skipped (the suite is
/// static; the unit test asserts the full 300 parse).
pub fn wac_positions() -> Vec<WacPosition> {
    WAC_EPD.lines().filter_map(parse_epd_line).collect()
}

fn parse_epd_line(line: &str) -> Option<WacPosition> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let (fields, rest) = line.split_once(" bm ")?;
    // The board description is exactly 4 fields; some entries carry extra EPD
    // opcodes (e.g. WAC.274's "am Rd6;") between them and `bm` — drop those.
    let fen_fields: Vec<&str> = fields.split_whitespace().take(4).collect();
    if fen_fields.len() != 4 {
        return None;
    }
    let fen = format!("{} 0 1", fen_fields.join(" "));
    let (moves, rest) = rest.split_once(';')?;
    let best_moves = moves.split_whitespace().map(str::to_string).collect();
    let id = rest
        .split_once("id \"")
        .and_then(|(_, id)| id.split_once('"'))
        .map_or_else(String::new, |(id, _)| id.to_string());
    Some(WacPosition {
        fen,
        best_moves,
        id,
    })
}

/// Whether `mv` (legal in `board`) is one of the accepted SAN `best_moves`.
pub fn move_matches_any(board: &Board, mv: Move, best_moves: &[String]) -> bool {
    best_moves.iter().any(|san| san_matches(board, mv, san))
}

/// Whether `mv` matches one SAN token. Handles castling (`O-O`/`O-O-O`),
/// promotions (`=Q`), piece letters, `x`, `+`/`#` suffixes, and file/rank
/// disambiguation hints. Legality/uniqueness is the suite's responsibility —
/// `mv` comes from the search, so only faithful matching matters here.
fn san_matches(board: &Board, mv: Move, san: &str) -> bool {
    let san = san.trim_end_matches(['+', '#']);
    let flags = mv.flags();

    if let Some(king_side) = match san {
        "O-O" | "0-0" => Some(true),
        "O-O-O" | "0-0-0" => Some(false),
        _ => None,
    } {
        return if king_side {
            flags == CASTLE_KINGSIDE
        } else {
            flags == CASTLE_QUEENSIDE
        };
    }

    let (san, promo) = match san.split_once('=') {
        Some((base, promo)) => (base, promo.chars().next()),
        None => (san, None),
    };
    match promo {
        Some(promo_char) => {
            if flags < PROMO_KNIGHT {
                return false;
            }
            let promoted = match (flags - PROMO_KNIGHT) % 4 {
                0 => 'N',
                1 => 'B',
                2 => 'R',
                _ => 'Q',
            };
            if promoted != promo_char.to_ascii_uppercase() {
                return false;
            }
        }
        None => {
            if flags >= PROMO_KNIGHT {
                return false;
            }
        }
    }

    let mut chars: Vec<char> = san.chars().collect();
    let piece = match chars.first() {
        Some('N') => Piece::Knight,
        Some('B') => Piece::Bishop,
        Some('R') => Piece::Rook,
        Some('Q') => Piece::Queen,
        Some('K') => Piece::King,
        _ => Piece::Pawn,
    };
    if piece != Piece::Pawn {
        chars.remove(0);
    }
    chars.retain(|&c| c != 'x');
    if chars.len() < 2 {
        return false;
    }

    if board.moving_piece(mv) != piece {
        return false;
    }

    let dest: String = chars[chars.len() - 2..].iter().collect();
    if square_name(mv.to_sq()) != dest {
        return false;
    }

    // Leftover leading chars are disambiguation hints on the origin square
    // (for pawns, the file of a capturing pawn, e.g. "exd5").
    let (from_file, from_rank) = {
        // `square_name` always yields exactly two ASCII chars (file, rank), so
        // this cannot fail; 9.0a names the invariant instead of two bare
        // `unwrap()`s — the only ones left in production code.
        let name = square_name(mv.from_sq());
        let mut it = name.chars();
        let file = it.next().expect("square_name yields a file char");
        let rank = it.next().expect("square_name yields a rank char");
        (file, rank)
    };
    chars[..chars.len() - 2].iter().all(|&hint| {
        if hint.is_ascii_digit() {
            hint == from_rank
        } else {
            hint == from_file
        }
    })
}

fn square_name(sq: Square) -> String {
    let file = b'a' + sq.file() as u8;
    let rank = b'1' + sq.rank() as u8;
    format!("{}{}", file as char, rank as char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_300_wac_positions_parse_and_are_legal() {
        let positions = wac_positions();
        assert_eq!(positions.len(), 300, "expected the full WAC suite");
        for pos in &positions {
            let board =
                Board::from_fen(&pos.fen).unwrap_or_else(|e| panic!("{} illegal: {e}", pos.id));
            assert!(!pos.best_moves.is_empty(), "{} has no bm", pos.id);
            // Every accepted SAN must match exactly one legal move — guards
            // both the matcher and the suite against typos/ambiguity.
            for san in &pos.best_moves {
                let matches = board
                    .generate_legal_moves()
                    .iter()
                    .filter(|&&mv| san_matches(&board, mv, san))
                    .count();
                assert_eq!(matches, 1, "{}: '{san}' matched {matches} moves", pos.id);
            }
        }
    }

    #[test]
    fn san_matcher_rejects_wrong_piece_and_destination() {
        let board =
            Board::from_fen("2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1").unwrap();
        let legal = board.generate_legal_moves();
        // Qg6 is WAC.001's answer: exactly one legal move matches, and it is
        // a queen move to g6.
        let matched: Vec<_> = legal
            .iter()
            .filter(|&&mv| san_matches(&board, mv, "Qg6"))
            .collect();
        assert_eq!(matched.len(), 1);
        assert_eq!(board.moving_piece(*matched[0]), Piece::Queen);
        assert_eq!(matched[0].to_sq().to_string(), "g6");
    }
}
