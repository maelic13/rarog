# Fritz 16 standalone HCE contract

Status: default standalone evaluator implemented and corpus-qualified;
C.1–C.8 retain their broader full-engine acceptance obligations.
Classification: evidence interpretation, implementation and deterministic
qualification. Maintainer explicitly requests the HCE dependency sequence in
Rust and integration into Rarog `dragon_nnue`, without committing or pushing.
This narrower dependency slice does not close the full-engine A/B gates.

Target SHA-256:
`264f2eede4c737f938c726e3c91e5ce07460e998110cf197abfc463dfa4622ae`.

| Address | Observed role / input / output |
| --- | --- |
| `0x14009e270` | Full evaluator, RCX points to caller-owned search-frame scratch; returns signed EAX, writes attack maps and auxiliary scores |
| `0x1400ca0f0` | Pawn/king/material-dependent cache reader/filler; returns pointer to 64-byte entry |
| `0x140001040..0x1400010b3` | Startup copy of 0x1800 bytes from `0x1402923b0` to `0x1404721b0`; recovered from saved independent linear assembly, absent from discovered-function C inventory |
| `0x1400010c0` | Piece-square sum, biased packed initial score `0x80008000`, castling terms `+0x640000,+0x140000,-0x640000,-0x140000` |
| `0x14005fdf0` | Packed material signature from popcounts; exact wrapping u64 constants in original pseudocode/assembly |
| `0x140088b30`, `0x1400c7060` | Search-root preparation of option/material adjustments before evaluator calls |

Representation: squares a1=0 through h8=63, piece bitboards interleaved by
color: WP,BP,WN,BN,WB,BB,WR,BR,WQ,BQ,WK,BK at
`0x1403c1110..0x1403c1168`; color occupancies at `0x1403c1100/1108`;
king squares at `0x1403c1170/1174`; occupancy at `0x1403c1178`;
castling at `0x1403c1180`; square mailbox at `0x1403c0e00`.
Confirm encodings using FEN producer `0x14005e850` and its initialized bytes.

The evaluator returns a white-perspective internal score; side-to-move
conversion belongs in the adapter and is to be checked against original callers.
Caller scratch offsets `+0x70` and `+0x74` influence passed-pawn race and tempo.
Root producers initialize these to side-to-move and +1/-1 respectively.
Default root material adjustments are -30/-90/-90/-150/-300 millipawns
per white pawn/knight/bishop/rook/queen and opposite signs for black;
UCI interventions and default option bytes must be checked separately.

First implementation strategy: mechanically translate saved native assembly
basic blocks into safe scalar Rust. Retain function IDs and instruction VAs;
do not execute the original text or call an original DLL at runtime. This
preserves branch/width semantics while making subsequent semantic refactoring
testable. The generator rejects unsupported instructions; no guessed fallback.
All mutable memory and pawn cache belong to one evaluator, allocated only at
configuration. Per-node inference must allocate nothing and take no locks.

Prospective qualification: compare full signed return and selected intermediate
fields against a direct original evaluator observation, with defaults frozen,
one original worker, random disabled, fresh cache and warm cache. Include both
turns, castling, sparse endings, opposite bishops, promotions and pawn structures.
Negative controls: change a parameter byte and require a score mismatch;
change arithmetic shift/sign-extension and require boundary-test rejection.
Search scores are not static-evaluation labels. No parity claim without a
qualified direct oracle. No playing-strength claim from static parity.

Qualification: 486 direct original static snapshots pass with cold/warm/reset
caches. Independent mailbox, bitboards, king squares, castling, packed PST,
material signature and pawn/king key match original snapshots. All 470 expanded
fixtures also match the default material adjustment read after the native root
search. A deliberately changed material-table entry produces a mismatch.
Native observations use the authorized read-only process-memory method in
../info-requests/C.8-static-oracle.md; search output is not a fixture label.
The first 16 fixtures and 470 expanded fixtures retain full input snapshots.

The null-move parser resets the root material-adjustment field to zero when
rebuilding its position. Snapshot parity therefore uses that exact observed
input, while a separate assertion checks the library's normal default adjustment
against the native root snapshot. Prior search is initialized on the target
side so retained caller-frame tempo is correct. The initial run that exposed
the opposite-side frame remains in work/hce-oracle-initial.

Remaining broader obligations: semantic names for every live table, nondefault
Fritz option interventions, exhaustive special-ending boundaries and original
search-dependent lazy paths. All instructions of the two full-evaluation
functions are translated; no guessed heuristic or placeholder replaces them.
Finite-corpus agreement does not establish exhaustive equivalence or strength.
Rarog owns its own draws, rule-50 handling, search corrections and tablebases.
