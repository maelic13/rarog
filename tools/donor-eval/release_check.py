"""The bridge DLL's values against the release binary's `eval` command, to
the command's printed precision. Usage:
  python -I release_check.py <sf11|sf16> <engine.exe> <parity.tsv> [count]
The TSV holds `fen<TAB>value`, the value from the side to move at root 0.
SF11 prints `Total evaluation: x (white side)` in PawnValueEg (213);
SF16 with NNUE off prints `Classical evaluation x (white side)` in
NormalizeToPawnValue (328). A value matches when the printed number is
the value's rounding to two decimals, give or take 0.01."""
import re
import subprocess
import sys

version, exe, tsv = sys.argv[1], sys.argv[2], sys.argv[3]
count = int(sys.argv[4]) if len(sys.argv) > 4 else 10**9
unit, pattern = {
    "sf11": (213, re.compile(r"Total evaluation:\s*([+-]?\d+\.\d+)")),
    "sf16": (328, re.compile(r"Classical evaluation\s+([+-]?\d+\.\d+)")),
}[version]

rows = []
with open(tsv, encoding="utf-8") as handle:
    for line in handle:
        fen, value = line.rstrip("\n").split("\t")
        rows.append((fen, int(value)))
rows = rows[:count]

commands = ["uci"]
if version == "sf16":
    commands.append("setoption name Use NNUE value false")
commands.append("isready")
for fen, _ in rows:
    commands += [f"position fen {fen}", "eval"]
commands.append("quit")
out = subprocess.run([exe], input="\n".join(commands) + "\n", capture_output=True,
                     text=True, timeout=1800).stdout
printed = [float(m.group(1)) for m in pattern.finditer(out)]
assert len(printed) == len(rows), (len(printed), len(rows))

bad = 0
worst = 0.0
for (fen, value), shown in zip(rows, printed):
    white = value if " w " in fen else -value
    expect = white / unit
    gap = abs(shown - expect)
    worst = max(worst, gap)
    if gap > 0.0051 + 0.01:
        bad += 1
        if bad <= 10:
            print(f"mismatch: DLL {white} ({expect:.4f}) printed {shown:.2f} {fen}")
print(f"release check {version}: {len(rows)} positions, {bad} outside the printed precision, "
      f"largest gap {worst:.4f} pawns")
sys.exit(1 if bad else 0)
