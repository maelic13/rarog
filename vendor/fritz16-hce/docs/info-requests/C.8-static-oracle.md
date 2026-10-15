# C.8 — Direct static-evaluation observation

| Field | Request |
| --- | --- |
| Affected gates | C.7/C.8 static evaluator and position adapter qualification |
| Original | Fritz 16, SHA-256 `264f2eede4c737f938c726e3c91e5ce07460e998110cf197abfc463dfa4622ae` |
| Addresses | Full evaluator `0x14009e270`; pawn cache `0x1400ca0f0`; UCI caller `0x1400e2b10`, call `0x1400e2c8a`, result store `0x1400e2ca5` to `0x1403c157c` |
| Question | Does the Rust translation return the exact original static score for identical board, option and caller-frame state? |
| Evidence checked | Both routines' saved assembly and C views; direct-call inventory; `0x1400e4cb0` accepts 0000; `0x1400e5030` toggles side on null; `0x1400e2b10` immediately calls the evaluator and stores its side-to-move return; PE layouts and table bytes; diagnostic eval returns no score |
| Proposed capture | `tools/capture_hce.py`: launch owned original via existing Session, initialize search state with depth 1, issue position plus null move, wait for readyok, use Windows ReadProcessMemory to read board, frame and option inputs plus stored evaluator score |
| Minimal output | Exact board/frame snapshots, signed static scores, configured options, original hash and raw UCI transcript; no debugger, injected code, process-memory writes, executable changes or thread-register changes |
| Bounds | At most 512 positions, 15-second individual response timeout, 180-second total capture budget; one worker, 16MB hash, random disabled; work/hce-oracle-runtime and fresh work/hce-oracle-* outputs; owned processes terminated on failure |
| Qualification | Confirm target side and mailbox; compare independently prepared Rust inputs and native snapshots; cold/warm/reset checks; a mutated constant must fail; retain all mismatches |
| Maintainer response | Authorized 2026-10-15: "Authorize read-only capture" |
| Result | Captured 16 initial positions, corrected retained caller-side state, then captured 16 directed and 470 expanded target-state fixtures. All 486 corrected fixtures match Rust; initial raw output retained. See ../experiments/HCE-001.md for hashes and limits. |

The depth-one search only initializes the original caller's state. Its search
score is not a static-evaluation fixture. The fixture value comes exclusively
from the assembly-identified return-value store after the null-move command.
