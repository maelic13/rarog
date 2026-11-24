use super::*;

/// A line printed inside the first millisecond reports `nps` from a
/// one-millisecond floor: 49 nodes read 49,000, not the raw count.
#[test]
fn nps_floors_the_elapsed_time_at_one_millisecond() {
    assert_eq!(nps(49, 0), 49_000);
    assert_eq!(nps(49, 1), 49_000);
    assert_eq!(nps(2_000, 4), 500_000);
    assert_eq!(nps(0, 0), 0);
}

/// An ordinary score is held below the tablebase band's display, so the
/// rule-50-downgraded mate never reads above a tablebase win.
#[test]
fn ordinary_scores_print_below_every_tablebase_win() {
    // The band's width moves with the ply cap; the display keeps every
    // tablebase win above every ordinary score, and both far above any
    // evaluation.
    let top = MAX_ORDINARY_CP;
    assert!(top > 19_000, "{top}");
    for material in [0, 17, 30, 58, 78, 100] {
        assert_eq!(
            format_score(TB_WIN_SCORE - 1, material),
            format!("cp {top}")
        );
        assert_eq!(
            format_score(-(TB_WIN_SCORE - 1), material),
            format!("cp -{top}")
        );
        assert_eq!(
            format_score(TB_WIN_SCORE, material),
            format!("cp {}", top + 1)
        );
        assert_eq!(format_score(TB_VALUE, material), "cp 20000");
        assert_eq!(
            format_score(-TB_WIN_SCORE, material),
            format!("cp -{}", top + 1)
        );
        assert_eq!(format_score(MATE_SCORE - 3, material), "mate 2");
        assert_eq!(format_score(-(MATE_SCORE - 4), material), "mate -2");
    }
}

/// The model is defined over every material a board can hold: `a` and
/// `b` stay positive, so the three rates are probabilities.
#[test]
fn the_model_is_defined_at_every_material() {
    for material in 0..=104 {
        let (a, b) = win_rate_params(material);
        assert!(a > 0.0 && b > 0.0, "material {material}: a {a}, b {b}");
    }
    assert_eq!(
        win_rate_params(5),
        win_rate_params(17),
        "held to the fitted range"
    );
    assert_eq!(win_rate_params(90), win_rate_params(78));
}

/// +100 is where the side wins half its games, at any material, and the
/// printed score never reverses the order of two scores.
#[test]
fn a_hundred_is_an_even_chance_of_winning_and_the_map_is_monotone() {
    for material in [17, 30, 45, 58, 78] {
        let (a, _) = win_rate_params(material);
        // The integer score nearest `a`, which the model calls an even chance.
        #[expect(clippy::cast_possible_truncation, reason = "a is a few hundred")]
        let even = a.round() as i32;
        let cp = ordinary_cp(even, material);
        assert!(
            (99..=101).contains(&cp),
            "material {material}: {even} prints {cp}"
        );
        let [win, _, _] = wdl(even, material);
        assert!((495..=505).contains(&win), "material {material}: win {win}");
        let mut last = i32::MIN;
        for score in (-3_000..=3_000).step_by(7) {
            let cp = ordinary_cp(score, material);
            assert!(
                cp >= last,
                "material {material}: {score} prints {cp} after {last}"
            );
            last = cp;
        }
        assert_eq!(ordinary_cp(0, material), 0);
    }
}

#[test]
fn the_rates_sum_to_a_thousand_mirror_and_decide_mates() {
    for material in [17, 58, 78] {
        for score in [-900, -150, -1, 0, 1, 150, 900] {
            let [win, draw, loss] = wdl(score, material);
            assert_eq!(win + draw + loss, 1000);
            assert_eq!(wdl(-score, material), [loss, draw, win]);
        }
        assert_eq!(wdl(MATE_SCORE - 5, material), [1000, 0, 0]);
        assert_eq!(wdl(-TB_VALUE, material), [0, 0, 1000]);
    }
}

/// `wdl` follows the score and its bound, as Stockfish prints it, and only
/// when asked for.
#[test]
fn the_rates_follow_the_score_only_when_shown() {
    let pv = [crate::board::Move::from_uci("e2e4").expect("valid move")];
    let record = |show_wdl| InfoRecord {
        depth: 9,
        seldepth: 12,
        multipv: 1,
        score: 90,
        bound: ScoreBound::Lower,
        nodes: 1_000,
        hashfull: 3,
        tb_hits: 0,
        elapsed_ms: 10,
        pv: &pv,
        material: 78,
        show_wdl,
    };
    let shown = iteration_line(&record(true));
    let [win, draw, loss] = wdl(90, 78);
    let expected = format!(
        " score {} lowerbound wdl {win} {draw} {loss} nodes ",
        format_score(90, 78)
    );
    assert!(shown.contains(&expected), "{shown}");
    let plain = iteration_line(&record(false));
    assert!(!plain.contains(" wdl "), "{plain}");
    assert!(
        plain.contains(&format!(
            " score {} lowerbound nodes ",
            format_score(90, 78)
        )),
        "{plain}"
    );
}

#[test]
fn a_root_with_no_legal_move_reports_mate_or_a_draw() {
    assert_eq!(
        report_line(&InfoReport::NoLegalMove { in_check: true }),
        "info depth 0 score mate 0"
    );
    assert_eq!(
        report_line(&InfoReport::NoLegalMove { in_check: false }),
        "info depth 0 score cp 0"
    );
}
