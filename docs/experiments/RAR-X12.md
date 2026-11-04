# RAR-X12 — Triumviratus's clock, protocol and thread defects, and its time-management rules

Indexed under *8. Cross-engine evidence imported from other engines › From Triumviratus* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Triumviratus evidence

The largest single result was a defect, not a mechanism. Unbuffered output
and a 15.6 ms clock cost +29.8 ± 7.4 at 5+0.05 with an identical tree. It
was found by timing `go nodes 1000` from outside: 6.15 ms, against Stockfish
19's 1.24.

Other defects cost it Elo or games:

- a 32-bit Zobrist generator, whose fix read +4.57 ± 3.56;
- re-sent options that reallocated, faking a 20-Elo socket gap;
- a quiescence TT depth of −1;
- a phantom en-passant square giving one position two keys;
- a 32-bit overflow in its correction decay;
- protocol defects before any `position`;
- a time manager that ignored the FEN's move number.

Its clock rules read +2 to +7 at 40+0.4, with wide intervals. Its defence
was the largest cell in its loss table against Stockfish 19: 33.8% of the
lost value sat in "defending with under 10% of the clock". TT16, an
XOR-verified 16-byte entry with a multiply-high index, read +6.30 ± 5.06
against its own cache-line-straddling layout. Items TT-1 to TT-10 and TP-1
to TP-6 in `analysis/triumviratus_review_2026-11-03.md`, which also lists the
fifteen checks.

## Rarog check

Run on `30593ba`, 2026-11-03 (the packet, §4).

- **Correct:** Zobrist rank 64, now a test (`b028d52`); output and clock
  overhead; re-sent options; quiescence TT depth; SEE pins and promotions;
  FEN keys; the time manager; the TT store; the SPSA schedule.
- **Measured from outside**, medians: `go nodes 1000` 0.55 ms (Stockfish 19
  0.93); `go movetime 100` 100.8 ms; a same-size `Hash` re-send 0.033 ms
  (Stockfish 19 1.15).
- **Found:** the correction bonus can overflow `i32` at extreme depth and
  residual (D.5.23), and a FEN halfmove clock above 255 ends the process
  (D.3.9).

## Possible Rarog implication

Rarog's protocol and clock surface is in good order. The wall-clock read is
D.4.1, done and clean: `go movetime` overshoots by under 1 ms, and in games
RAR-R30 holds 1.2% of the charged time at one thread. The paths read correct
but untested become D.5.17 tests. The time-management rules are small gains
for H.3. The loss table is the instrument D.4.5, which E.1 re-reads.

## Existing PLAN coverage

D.2, D.3.9, D.5.17, D.5.23, D.6.12, D.4.1, D.4.5, E.1, G.1, H.3
