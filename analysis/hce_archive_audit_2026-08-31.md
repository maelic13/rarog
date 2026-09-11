# HCE self-play archive audit — 2026-08-31

## Decision

The two existing archives are internally consistent and suitable for one
qualified self-play-WDL fit, but they cannot supply the former 3,000,000-row
equal-phase target. The frozen production contract is therefore **2,300,000
train + 127,778 validation + 127,778 test**, with five equal phase reservoirs.
The opening reservoir is the limiting population.

This audit wrote no dataset. `tools/texel/fit_complete.ps1` repeats it, then
publishes `tools/texel/data/hce-v2` atomically and records the three output
hashes before fitting.

## Reproduction

```powershell
python tools/texel/extract_parallel.py `
    tools/texel/data/selfplay-p1025a-zero-n8000-s1-g20000.pgn `
    tools/texel/data/selfplay-p1025a-zero-n8000-s20001-g580000.pgn `
    --out-dir tools/texel/data/hce-v2 --target-train 2300000 `
    --jobs 14 --audit-only
```

Input hashes:

| Artifact | SHA-256 |
|---|---|
| `selfplay-p1025a-zero-n8000-s1-g20000.pgn` | `054CF7C66FC2B1F888B4E7881DDDD694AB21425515D07CE2F3A6ECA41CBFDFAB` |
| `selfplay-p1025a-zero-n8000-s20001-g580000.pgn` | `4600D8DFB632FD1414EA5622E155079A9383DEE334987225A547BE4BA2EB9F20` |
| pilot manifest | `B3CE9056D1C6EF93182CFB4AFA8E4E095B3C4BECBCBF66DE0149D11DE759CC3C` |
| continuation manifest | `4F4A494C03A6DE99D0D7FD1FF4F106DD0A1EDD9441411B632EF7ECE43EEECF59` |

Both manifests bind the same engine (`74d4426ff3c4`, binary SHA-256
`9AC35CC26D954E55E394E5AAE5FE4FCE09E6F2D3ECE0DF135F1009FF0917E0C9`),
8,000 nodes/move, book hash, shuffle seed 10403 and
`datagen-v1` adjudication profile. Their book ranges are disjoint: 1–20,000
and 20,001–600,000.

The bound `beast_seed.epd` contains exactly **750,000 lines, 750,000 unique
four-field FENs and zero malformed or duplicate entries**. Its SHA-256 is
`B91C756ADCC7A4B96CEA502A7775B128DDE6C425F6A2B4FC6381E603610B2C7F`.
Thus the 600,000 games are independent opening starts rather than repeated
games hidden behind a large nominal file. `datagen.ps1` and the fit runner now
enforce this mechanically.

## Measured content

| Check | Result |
|---|---:|
| Independent / recorded starts | 600,000 / 600,000 |
| Replayed starts / parse errors | 0 / 0 |
| Raw / unique eligible rows | 6,520,640 / 6,501,318 |
| Quiet-filter rejects / games with no retained row | 1,178,550 / 8,430 |
| Results W / B / draw | 169,257 / 144,595 / 286,148 |
| Natural checkmates | 6,428 |
| Mean game length | 66.43 plies |
| Unique-row labels 1 / 0 / 0.5 | 1,883,651 / 1,647,779 / 2,969,888 |
| Distinct non-king material signatures | 26,935 |
| Both queens / one queen / no queens / pawn-only rows | 1,839,608 / 115,012 / 4,354,229 / 192,469 |

Termination cross-check:

| Termination | White wins | Black wins | Draws | Total |
|---|---:|---:|---:|---:|
| `adjudication` | 165,895 | 141,529 | 5,494 | 312,918 |
| `normal` | 3,362 | 3,066 | 280,654 | 287,082 |

`datagen-v1` is deliberate and provenance-bound: resign requires both engines
above 600 cp for three moves; draw requires eight moves below 10 cp after move
40. Regenerating 600,000 games without adjudication is not justified before
the current corpus receives its offline and game verdict. If transfer fails,
a fresh-current/no-adjudication corpus is a possible *registered changed-data
hypothesis*, not an automatic retry.

## Capacity and quotas

The former 3,000,000 target required 600,000 train openings, but only 460,752
exist; it is mechanically impossible on these archives. At 2,300,000, every
quota passes:

| Split | Rows | Per-phase quotas | Limiting opening yield |
|---|---:|---:|---:|
| Train | 2,300,000 | 460,000 each | 460,752 |
| Validation | 127,778 | 25,556 / 25,556 / 25,556 / 25,555 / 25,555 | 25,920 |
| Test | 127,778 | 25,556 / 25,556 / 25,556 / 25,555 / 25,555 | 25,772 |

The extraction is deterministic for the fixed inputs, seed and 16 byte ranges;
the narrow opening margins are therefore reproducible rather than statistical
estimates. Publication still fails closed if any exact quota or provenance
check changes.
