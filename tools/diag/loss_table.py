#!/usr/bin/env python3
"""The loss table by cell (PLAN D.4.5): where an engine loses its points.

From games already played (Colosseum CLI run directories), a judge engine
scores every move at fixed nodes: its evaluation before the move minus its
evaluation after it, from the mover's side, in expected score (from the
judge's win-draw-loss) and in centipawns. The moves are tabulated for the
engine and, as the control, for its opponents, by position class (the
judge's score before the move), clock left before the move and men on the
board. The engine's worst moves are then searched again by the engine at
multiples of the nodes it spent in the game, to split errors that more
nodes repair (the clock's) from errors that stay (the evaluation's).

  python tools/diag/loss_table.py sample --run tools/results/pool2-rr \\
      --engine "Rarog C.11 head" --out sample.tsv
  python tools/diag/loss_table.py judge sample.tsv --judge <stockfish.exe> \\
      --nodes 1000000 --workers 14 --out judged.tsv
  python tools/diag/loss_table.py table judged.tsv [--by class,clock,men]
  python tools/diag/loss_table.py research judged.tsv --exe <engine.exe> \\
      --judge <stockfish.exe> --min-loss 0.10 --workers 14 --out research.tsv
  python tools/diag/loss_table.py split research.tsv

`sample` takes every game the engine lost, every draw it spoiled (for at
least `--spoil-moves` consecutive moves of each side, the engine's printed
score was at least `--spoil-cp` and its opponent's at most minus that: both
saw the engine ahead) and the twin of each (the pair's other game: the same
opening with colours reversed) as the selection control; `--all` takes
every game the engine played.

`judge` runs one judge per worker with one thread, a fixed hash and
`ucinewgame` before each game, whose positions it searches in order with the
game's moves, so a game's records do not depend on the workers or the
order. A final position the rules decide (mate, stalemate, insufficient
material, a threefold or the fifty-move rule) is scored by the rule. Each
game's records end with a `# game` line; a rerun on the same output keeps
finished games, drops a partial one and continues. Ctrl+C stops a judge or
research run within a second; the judge's finished games stay.

A game's losses telescope: the engine's summed loss minus its opponent's is
the engine's expected score at the first position minus the judge's at the
last, so the table attributes each game's points to cells. Where an
adjudication ended the game, the judge's last expected score minus the
result is carried by no move. `table` prints both over the read and fails
when a game's losses do not telescope.

`research` re-searches each engine move whose judged loss is at least
`--min-loss`, from a fresh table at 1, 4 and 16 times the game's nodes, and
judges the position before the move, the game's move and each new move
afresh. `split` reads the verdicts: unconfirmed (the fresh judge does not
see the error), not reproduced (a fresh search at the game's nodes avoids
it), clock at 4x or 16x (more nodes avoid it), or persists.
"""

from __future__ import annotations

import argparse
import collections
import dataclasses
import hashlib
import multiprocessing
import os
import pathlib
import re
import signal
import subprocess
import sys
import time

import chess
import chess.pgn

MATE_CP = 10000
CP_CAP = 300
CP_ERROR = 50
CLASS_CP = 50
CLOCK_BANDS = (10, 25, 50)
MEN_BANDS = (9, 15, 23)
MULTIPLES = (1, 4, 16)
SAMPLE_MAGIC = "# loss_table sample v1"
JUDGED_MAGIC = "# loss_table judged v2"
RESEARCH_MAGIC = "# loss_table research v1"
RECORD_FIELDS = ("run", "game", "ply", "mover", "name", "opponent", "colour", "move",
                 "men", "clock_ms", "clock_frac", "game_s", "game_n", "game_t",
                 "cp_before", "es_before", "cp_after", "es_after", "loss_cp", "loss_es",
                 "reason", "result", "fen")


# --- games -----------------------------------------------------------------

@dataclasses.dataclass(frozen=True)
class Ply:
    uci: str
    score: str | None
    nodes: int | None
    ms: int | None


@dataclasses.dataclass(frozen=True)
class Game:
    run: str
    number: int
    pair: int
    pair_game: int
    white: str
    black: str
    result: str
    start_fen: str
    clock: dict
    plies: tuple


def parse_comment(comment: str) -> tuple[str | None, int | None, int | None]:
    """The printed score, nodes and charged milliseconds of one move."""
    tokens = dict(t.split("=", 1) for t in comment.split() if "=" in t)
    score = tokens.get("s")
    if score is not None and not re.fullmatch(r"#?-?\d+", score):
        raise ValueError(f"unreadable score {score!r} in {comment!r}")
    nodes = int(tokens["n"]) if "n" in tokens else None
    ms = None
    if "t" in tokens:
        match = re.fullmatch(r"(\d+)ms", tokens["t"])
        if not match:
            raise ValueError(f"unreadable time {tokens['t']!r} in {comment!r}")
        ms = int(match.group(1))
    return score, nodes, ms


def parse_time_control(tag: str) -> dict:
    """Base and increment in milliseconds per colour."""
    match = re.fullmatch(r"white (\d+)ms\+(\d+)ms; black (\d+)ms\+(\d+)ms", tag.strip())
    if match:
        wb, wi, bb, bi = (int(g) for g in match.groups())
        return {"w": (wb, wi), "b": (bb, bi)}
    match = re.fullmatch(r"(\d+(?:\.\d+)?)\+(\d+(?:\.\d+)?)", tag.strip())
    if match:
        base, inc = (round(float(g) * 1000) for g in match.groups())
        return {"w": (base, inc), "b": (base, inc)}
    raise ValueError(f"unreadable TimeControl {tag!r}")


def score_value(score: str | None) -> float | None:
    """A printed score from the mover's side; a mate is infinite."""
    if score is None:
        return None
    if score.startswith("#"):
        return float("inf") if not score.startswith("#-") else float("-inf")
    return float(score)


def read_game(handle, run: str) -> Game | None:
    game = chess.pgn.read_game(handle)
    if game is None:
        return None
    if game.errors:
        raise ValueError(f"{run}: game {game.headers.get('GameNumber')}: {game.errors[0]}")
    h = game.headers
    plies = []
    for node in game.mainline():
        score, nodes, ms = parse_comment(node.comment)
        plies.append(Ply(node.move.uci(), score, nodes, ms))
    return Game(run, int(h["GameNumber"]), int(h.get("PairNumber", 0)),
                int(h.get("PairGame", 0)), h["White"], h["Black"], h["Result"],
                h.get("FEN", chess.STARTING_FEN), parse_time_control(h["TimeControl"]),
                tuple(plies))


def scan(path: pathlib.Path, run: str, wanted=None):
    """Yield the games of one games.pgn; with `wanted(headers)`, parse only those."""
    with open(path, encoding="utf-8", newline="") as handle:
        while True:
            offset = handle.tell()
            headers = chess.pgn.read_headers(handle)
            if headers is None:
                return
            if wanted is not None and not wanted(headers):
                continue
            end = handle.tell()
            handle.seek(offset)
            game = read_game(handle, run)
            handle.seek(end)
            yield game


def sha256(path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def engine_colour(game: Game, engine: str) -> str:
    if game.white == engine:
        return "w"
    if game.black == engine:
        return "b"
    raise ValueError(f"{engine!r} did not play game {game.number}")


def outcome(game: Game, colour: str) -> float:
    """The engine's points in the game."""
    if game.result == "1/2-1/2":
        return 0.5
    if game.result not in ("1-0", "0-1"):
        raise ValueError(f"game {game.number}: result {game.result!r}")
    return 1.0 if (game.result == "1-0") == (colour == "w") else 0.0


def spoiled(game: Game, colour: str, spoil_cp: int, spoil_moves: int) -> bool:
    """A draw in which both engines' printed scores had `colour` ahead for
    `spoil_moves` consecutive moves of each; unscored plies are skipped."""
    if game.result != "1/2-1/2":
        return False
    board_colour = "w" if chess.Board(game.start_fen).turn == chess.WHITE else "b"
    run = 0
    for i, ply in enumerate(game.plies):
        value = score_value(ply.score)
        if value is None:
            continue
        mover = board_colour if i % 2 == 0 else ("b" if board_colour == "w" else "w")
        agrees = value >= spoil_cp if mover == colour else value <= -spoil_cp
        run = run + 1 if agrees else 0
        if run >= 2 * spoil_moves:
            return True
    return False


def select(games: list[Game], engine: str, spoil_cp: int, spoil_moves: int,
           every: bool) -> list[tuple[Game, str]]:
    """The sample: (game, reason) in run and game order."""
    chosen: dict[tuple[str, int], str] = {}
    by_key = {(g.run, g.number): g for g in games}
    by_pair = collections.defaultdict(list)
    for game in games:
        by_pair[(game.run, game.pair)].append(game)
        colour = engine_colour(game, engine)
        if every:
            chosen[(game.run, game.number)] = "all"
        elif outcome(game, colour) == 0.0:
            chosen[(game.run, game.number)] = "loss"
        elif spoiled(game, colour, spoil_cp, spoil_moves):
            chosen[(game.run, game.number)] = "spoiled"
    if not every:
        for key in list(chosen):
            for twin in by_pair[(key[0], by_key[key].pair)]:
                chosen.setdefault((twin.run, twin.number), "twin")
    return [(by_key[key], chosen[key]) for key in sorted(chosen)]


def cmd_sample(args) -> int:
    games: list[Game] = []
    header = [SAMPLE_MAGIC]
    for run_dir in args.run:
        run_dir = pathlib.Path(run_dir)
        pgn = run_dir / "games.pgn"
        run = run_dir.name
        header.append(f"# run {run} {run_dir.resolve()} games.pgn sha256 {sha256(pgn)}")
        # A draw's spoiling needs its printed scores and a twin is known only
        # once its pair is seen, so every game of the engine is parsed here;
        # the judge parses only the sample.
        run_games = list(scan(pgn, run, lambda h: args.engine in (h.get("White"), h.get("Black"))))
        if not run_games:
            names = sorted({n for h in _headers(pgn) for n in (h["White"], h["Black"])})
            raise SystemExit(f"{run}: no game of {args.engine!r}; names: {', '.join(names)}")
        games.extend(run_games)
    header.append(f"# engine {args.engine}")
    header.append(f"# rules spoil_cp {args.spoil_cp} spoil_moves {args.spoil_moves}"
                  f" all {int(args.all)}")
    sample = select(games, args.engine, args.spoil_cp, args.spoil_moves, args.all)
    lines = header + ["\t".join(("run", "game", "pair", "white", "black", "result",
                                 "reason", "engine_colour", "plies"))]
    for game, reason in sample:
        lines.append("\t".join(str(v) for v in (
            game.run, game.number, game.pair, game.white, game.black, game.result,
            reason, engine_colour(game, args.engine), len(game.plies))))
    pathlib.Path(args.out).write_text("\n".join(lines) + "\n", encoding="utf-8")
    counts = collections.Counter(reason for _, reason in sample)
    plies = sum(len(g.plies) for g, _ in sample)
    print(f"{len(games)} games of {args.engine}; sample {len(sample)} games"
          f" ({', '.join(f'{k} {v}' for k, v in sorted(counts.items()))}), {plies} plies")
    return 0


def _headers(pgn: pathlib.Path):
    with open(pgn, encoding="utf-8", newline="") as handle:
        while (h := chess.pgn.read_headers(handle)) is not None:
            yield h


@dataclasses.dataclass(frozen=True)
class Sample:
    path: pathlib.Path
    sha: str
    engine: str
    runs: dict
    rows: list


def read_sample(path) -> Sample:
    path = pathlib.Path(path)
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != SAMPLE_MAGIC:
        raise SystemExit(f"{path}: not a loss_table sample")
    runs, engine, rows, columns = {}, None, [], None
    for line in lines[1:]:
        if line.startswith("# run "):
            _, _, run, rest = line.split(" ", 3)
            location, _, _, digest = rest.rsplit(" ", 3)
            runs[run] = (pathlib.Path(location), digest)
        elif line.startswith("# engine "):
            engine = line[len("# engine "):]
        elif line.startswith("#"):
            continue
        elif columns is None:
            columns = line.split("\t")
        else:
            rows.append(dict(zip(columns, line.split("\t"))))
    return Sample(path, sha256(path), engine, runs, rows)


def sample_games(sample: Sample) -> list[tuple[Game, str]]:
    """The sampled games, checked against the hashes the sample recorded."""
    out = []
    for run, (location, digest) in sample.runs.items():
        pgn = location / "games.pgn"
        if sha256(pgn) != digest:
            raise SystemExit(f"{pgn} changed since the sample was drawn")
        wanted = {int(r["game"]): r["reason"] for r in sample.rows if r["run"] == run}
        games = {g.number: g for g in scan(pgn, run, lambda h: int(h["GameNumber"]) in wanted)}
        missing = set(wanted) - set(games)
        if missing:
            raise SystemExit(f"{run}: sampled games missing: {sorted(missing)[:5]}")
        out.extend((games[n], wanted[n]) for n in sorted(wanted))
    return out


# --- UCI -------------------------------------------------------------------

@dataclasses.dataclass(frozen=True)
class Eval:
    """A position's value for the side to move."""
    cp: int
    es: float
    best: str | None
    terminal: bool = False


class Uci:
    def __init__(self, exe: str, options: list[tuple[str, str]]):
        self.exe = exe
        # An engine in the console's group would die of Ctrl+C under a live
        # worker; in its own group it quits when its worker's pipe closes.
        group = ({"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP} if os.name == "nt"
                 else {"start_new_session": True})
        self.proc = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                     stderr=subprocess.DEVNULL, text=True, bufsize=1,
                                     cwd=os.path.dirname(os.path.abspath(exe)), **group)
        self.name = None
        for line in self.command("uci", "uciok"):
            if line.startswith("id name "):
                self.name = line[len("id name "):].strip()
        for name, value in options:
            self.send(f"setoption name {name} value {value}")
        self.command("isready", "readyok")

    def send(self, line: str) -> None:
        self.proc.stdin.write(line + "\n")
        self.proc.stdin.flush()

    def command(self, line: str, until: str) -> list[str]:
        self.send(line)
        lines = []
        while True:
            out = self.proc.stdout.readline()
            if not out:
                raise RuntimeError(f"{self.exe} exited during {line!r}")
            out = out.strip()
            lines.append(out)
            if out.split(" ", 1)[0] == until:
                return lines

    def new_game(self) -> None:
        self.send("ucinewgame")
        self.command("isready", "readyok")

    def search(self, fen: str, moves, nodes: int, need_wdl: bool) -> Eval:
        position = f"position fen {fen}" + (f" moves {' '.join(moves)}" if moves else "")
        self.send(position)
        lines = self.command(f"go nodes {nodes}", "bestmove")
        return parse_search(lines, need_wdl)

    def close(self) -> None:
        try:
            self.send("quit")
            self.proc.wait(timeout=10)
        except (OSError, subprocess.TimeoutExpired):
            self.proc.kill()


def parse_search(lines: list[str], need_wdl: bool) -> Eval:
    """The last exact score of the principal line, with its WDL."""
    best = lines[-1].split()[1]
    chosen = None
    for line in lines:
        tokens = line.split()
        if not tokens or tokens[0] != "info" or "score" not in tokens:
            continue
        if "multipv" in tokens and tokens[tokens.index("multipv") + 1] != "1":
            continue
        bound = "lowerbound" in tokens or "upperbound" in tokens
        if chosen is None or not bound or chosen[1]:
            chosen = (tokens, bound)
    if chosen is None:
        raise RuntimeError(f"no score before {lines[-1]!r}")
    tokens = chosen[0]
    i = tokens.index("score")
    kind, value = tokens[i + 1], int(tokens[i + 2])
    cp = value if kind == "cp" else (MATE_CP - abs(value)) * (1 if value > 0 else -1)
    if "wdl" in tokens:
        j = tokens.index("wdl")
        wins, draws, losses = (int(t) for t in tokens[j + 1:j + 4])
        es = (wins + draws / 2) / (wins + draws + losses)
    elif need_wdl:
        raise RuntimeError("the judge prints no wdl: it needs UCI_ShowWDL")
    else:
        es = float("nan")
    return Eval(cp, es, best)


def terminal(board: chess.Board) -> Eval | None:
    """A final position the rules decide, for the side to move."""
    if board.is_checkmate():
        return Eval(-MATE_CP, 0.0, None, True)
    if (board.is_stalemate() or board.is_insufficient_material()
            or board.is_repetition(3) or board.is_fifty_moves()):
        return Eval(0, 0.5, None, True)
    return None


# --- judge -----------------------------------------------------------------

def clocks(game: Game) -> list[int]:
    """Each ply's clock before the move, in milliseconds."""
    turn = "w" if chess.Board(game.start_fen).turn == chess.WHITE else "b"
    left = {c: game.clock[c][0] for c in "wb"}
    out = []
    for ply in game.plies:
        out.append(left[turn])
        left[turn] += game.clock[turn][1] - (ply.ms or 0)
        turn = "b" if turn == "w" else "w"
    return out


def judge_game(game: Game, reason: str, engine: str, evaluate) -> list[dict]:
    """Every move's record; `evaluate(fen, moves)` scores a position the rules
    do not decide, for the side to move."""
    board = chess.Board(game.start_fen)
    moves = [p.uci for p in game.plies]
    evals, fens, men = [], [], []
    for k in range(len(moves) + 1):
        fens.append(board.fen())
        men.append(chess.popcount(board.occupied))
        final = terminal(board) if k == len(moves) else None
        evals.append(final or evaluate(game.start_fen, moves[:k]))
        if k < len(moves):
            board.push_uci(moves[k])
    clock = clocks(game)
    colour = engine_colour(game, engine)
    opponent = game.black if colour == "w" else game.white
    result = outcome(game, colour)
    first = "w" if chess.Board(game.start_fen).turn == chess.WHITE else "b"
    records = []
    for k, ply in enumerate(game.plies):
        mover_colour = first if k % 2 == 0 else ("b" if first == "w" else "w")
        before, after = evals[k], evals[k + 1]
        cp_after, es_after = -after.cp, 1.0 - after.es
        base = game.clock[mover_colour][0]
        records.append({
            "run": game.run, "game": game.number, "ply": k,
            "mover": "engine" if mover_colour == colour else "opponent",
            "name": game.white if mover_colour == "w" else game.black,
            "opponent": opponent, "colour": mover_colour, "move": ply.uci,
            "men": men[k], "clock_ms": clock[k], "clock_frac": round(clock[k] / base, 4),
            "game_s": ply.score if ply.score is not None else "-",
            "game_n": ply.nodes if ply.nodes is not None else "-",
            "game_t": ply.ms if ply.ms is not None else "-",
            "cp_before": before.cp, "es_before": round(before.es, 4),
            "cp_after": cp_after, "es_after": round(es_after, 4),
            "loss_cp": before.cp - cp_after, "loss_es": round(before.es - es_after, 4),
            "reason": reason, "result": result, "fen": fens[k],
        })
    return records


_WORKER: dict = {}


def _ignore_interrupts() -> None:
    """Ctrl+C is the parent's to handle: a worker that dies of it loses its
    task, and the pool waits for that result forever."""
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    if hasattr(signal, "SIGBREAK"):
        signal.signal(signal.SIGBREAK, signal.SIG_IGN)


def results(iterator):
    """A pool's results, waited for in short steps: on Windows a bare wait
    on a pool's results never sees Ctrl+C."""
    while True:
        try:
            yield iterator.next(timeout=0.5)
        except multiprocessing.TimeoutError:
            continue
        except StopIteration:
            return


def _judge_init(judge: str, options, nodes: int) -> None:
    _ignore_interrupts()
    _WORKER["judge"] = Uci(judge, options)
    _WORKER["nodes"] = nodes


def _judge_one(item):
    game, reason, engine = item
    judge = _WORKER["judge"]
    judge.new_game()
    nodes = _WORKER["nodes"]
    records = judge_game(game, reason, engine,
                         lambda fen, moves: judge.search(fen, moves, nodes, True))
    return game.run, game.number, len(game.plies), records


def judge_options(hash_mb: int) -> list[tuple[str, str]]:
    return [("Threads", "1"), ("Hash", str(hash_mb)), ("UCI_ShowWDL", "true")]


def format_record(record: dict) -> str:
    return "\t".join(str(record[f]) for f in RECORD_FIELDS)


def resume(out: pathlib.Path, header: list[str]) -> set:
    """Keep the finished games of an earlier run on the same output."""
    if not out.exists():
        out.write_text("\n".join(header + ["\t".join(RECORD_FIELDS)]) + "\n", encoding="utf-8")
        return set()
    lines = out.read_text(encoding="utf-8").splitlines()
    if lines[:len(header)] != header:
        raise SystemExit(f"{out} was written by another sample, judge or setting")
    kept, pending, done = lines[:len(header) + 1], [], set()
    for line in lines[len(header) + 1:]:
        parts = line.split(" ")
        if len(parts) == 6 and parts[:2] == ["#", "game"] and parts[4] == "plies":
            _, _, run, number, _, _ = parts
            kept.extend(pending)
            kept.append(line)
            pending = []
            done.add((run, int(number)))
        else:
            pending.append(line)
    out.write_text("\n".join(kept) + "\n", encoding="utf-8")
    return done


def cmd_judge(args) -> int:
    sample = read_sample(args.sample)
    games = sample_games(sample)
    probe = Uci(args.judge, judge_options(args.hash))
    judge_name = probe.name
    probe.close()
    header = [JUDGED_MAGIC,
              f"# sample {sample.path.resolve()} sha256 {sample.sha}",
              f"# engine {sample.engine}",
              f"# judge {pathlib.Path(args.judge).resolve()} sha256 {sha256(args.judge)}"
              f" id {judge_name}",
              f"# nodes {args.nodes} hash {args.hash} threads 1"]
    out = pathlib.Path(args.out)
    done = resume(out, header)
    todo = [(g, r, sample.engine) for g, r in games if (g.run, g.number) not in done]
    print(f"{len(games)} games, {len(done)} already judged, {len(todo)} to judge,"
          f" {sum(len(g.plies) + 1 for g, _, _ in todo)} positions", flush=True)
    start, finished = time.time(), 0
    with open(out, "a", encoding="utf-8", newline="\n") as sink, multiprocessing.Pool(
            args.workers, _judge_init, (args.judge, judge_options(args.hash), args.nodes)) as pool:
        try:
            for run, number, plies, records in results(pool.imap_unordered(_judge_one, todo)):
                sink.write("".join(format_record(r) + "\n" for r in records)
                           + f"# game {run} {number} plies {plies}\n")
                sink.flush()
                finished += 1
                if finished % args.progress == 0 or finished == len(todo):
                    rate = (time.time() - start) / finished
                    print(f"{finished}/{len(todo)} games, {rate:.1f} s a game,"
                          f" {rate * (len(todo) - finished) / 3600:.2f} h left", flush=True)
        except KeyboardInterrupt:
            pool.terminate()
            print(f"stopped: {len(done) + finished} of {len(games)} games judged and kept;"
                  " the same command continues", flush=True)
            return 130
    return 0


# --- table -----------------------------------------------------------------

@dataclasses.dataclass
class Judged:
    header: dict
    records: list
    games: set


def read_judged(path) -> Judged:
    lines = pathlib.Path(path).read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != JUDGED_MAGIC:
        raise SystemExit(f"{path}: not a judged file")
    header, records, games, columns, pending = {}, [], set(), None, []
    for line in lines[1:]:
        if line.startswith("# game "):
            _, _, run, number, _, plies = line.split(" ")
            if len(pending) != int(plies):
                raise SystemExit(f"{path}: game {run} {number} has {len(pending)} of {plies} records")
            records.extend(pending)
            pending = []
            games.add((run, int(number)))
        elif line.startswith("# "):
            key, _, value = line[2:].partition(" ")
            header[key] = value
        elif columns is None:
            columns = line.split("\t")
            if tuple(columns) != RECORD_FIELDS:
                raise SystemExit(f"{path}: unexpected columns")
        else:
            values = dict(zip(columns, line.split("\t")))
            for key in ("game", "ply", "men", "clock_ms", "cp_before", "cp_after", "loss_cp"):
                values[key] = int(values[key])
            for key in ("clock_frac", "es_before", "es_after", "loss_es", "result"):
                values[key] = float(values[key])
            pending.append(values)
    if pending:
        raise SystemExit(f"{path}: {len(pending)} records after the last finished game")
    return Judged(header, records, games)


CLASSES = ("defending", "level", "ahead")
CLOCK_LABELS = ([f"<{CLOCK_BANDS[0]}%"] + [f"{a}-{b}%" for a, b in zip(CLOCK_BANDS, CLOCK_BANDS[1:])]
                + [f">={CLOCK_BANDS[-1]}%"])
MEN_LABELS = ([f"<={MEN_BANDS[0]}"] + [f"{a + 1}-{b}" for a, b in zip(MEN_BANDS, MEN_BANDS[1:])]
              + [f">={MEN_BANDS[-1] + 1}"])


def cell(record: dict, dim: str, class_cp: int) -> tuple[int, str]:
    """The record's label in one dimension, with its rank for ordering."""
    if dim == "class":
        cp = record["cp_before"]
        index = 0 if cp <= -class_cp else 2 if cp >= class_cp else 1
        return index, CLASSES[index]
    if dim == "clock":
        pct = record["clock_frac"] * 100
        index = next((i for i, edge in enumerate(CLOCK_BANDS) if pct < edge), len(CLOCK_BANDS))
        return index, CLOCK_LABELS[index]
    if dim == "men":
        index = next((i for i, edge in enumerate(MEN_BANDS) if record["men"] <= edge), len(MEN_BANDS))
        return index, MEN_LABELS[index]
    if dim in ("opponent", "reason", "name"):
        return 0, record[dim]
    raise SystemExit(f"unknown dimension {dim!r}")


def capped(loss_cp: int) -> int:
    return max(-CP_CAP, min(CP_CAP, loss_cp))


def tabulate(records: list[dict], dims: list[str], class_cp: int = CLASS_CP) -> list[dict]:
    totals = collections.Counter()
    groups = collections.defaultdict(list)
    for r in records:
        totals[r["mover"]] += r["loss_es"]
        groups[(r["mover"] != "engine",) + tuple(cell(r, d, class_cp) for d in dims)].append(r)
    rows = []
    for key in sorted(groups):
        group = groups[key]
        side = group[0]["mover"]
        points = sum(r["loss_es"] for r in group)
        errors = sum(r["loss_cp"] >= CP_ERROR for r in group)
        rows.append({
            "side": side, **{d: label for d, (_, label) in zip(dims, key[1:])}, "moves": len(group),
            "games": len({(r["run"], r["game"]) for r in group}),
            "mean_cp": sum(capped(r["loss_cp"]) for r in group) / len(group),
            "errors": errors, "error_share": errors / len(group),
            "points": points, "point_share": points / totals[side] if totals[side] else 0.0,
            "mean_mpts": 1000 * points / len(group),
        })
    return rows


@dataclasses.dataclass(frozen=True)
class Identity:
    """The engine's points over a read, all from its side. `net` is its summed
    loss minus its opponents'; `to_final` its expected score at each game's
    first position minus the judge's at the last; `to_result` the first minus
    its result; `worst` the largest one game's |net - to_final|."""
    net: float
    to_final: float
    to_result: float
    worst: float


IDENTITY_TOLERANCE = 1e-6


def identity(records: list[dict]) -> Identity:
    """A game's losses telescope to its first position minus its last; what an
    adjudication leaves between the last position and the result is carried
    by no move."""
    by_game = collections.defaultdict(list)
    for r in records:
        by_game[(r["run"], r["game"])].append(r)
    net = to_final = to_result = worst = 0.0
    for moves in by_game.values():
        moves.sort(key=lambda r: r["ply"])
        game_net = sum(r["loss_es"] if r["mover"] == "engine" else -r["loss_es"] for r in moves)
        first, last = moves[0], moves[-1]
        start = first["es_before"] if first["mover"] == "engine" else 1 - first["es_before"]
        end = last["es_after"] if last["mover"] == "engine" else 1 - last["es_after"]
        net += game_net
        to_final += start - end
        to_result += start - last["result"]
        worst = max(worst, abs(game_net - (start - end)))
    return Identity(net, to_final, to_result, worst)


def cmd_table(args) -> int:
    judged = read_judged(args.judged)
    records = [r for r in judged.records if not args.reason or r["reason"] in args.reason]
    dims = [d for d in args.by.split(",") if d] if args.by else []
    rows = tabulate(records, dims, args.class_cp)
    ident = identity(records)
    print(f"{judged.header.get('judge', '?')}; {judged.header.get('nodes', '?')}")
    print(f"{len({(r['run'], r['game']) for r in records})} games, {len(records)} moves;"
          f" the engine's loss minus its opponents' {ident.net:.3f} points, its start minus"
          f" the judge's last {ident.to_final:.3f} (largest game difference"
          f" {ident.worst:.1e}); its start minus its result {ident.to_result:.3f}, the"
          f" judge's last minus the result {ident.to_result - ident.to_final:.3f} carried"
          f" by no move")
    head = ["side"] + dims + ["moves", "games", "mean_cp", "errors", "error_share",
                              "points", "point_share", "mean_mpts"]
    widths = [max(len(h), *(len(_fmt(r[h])) for r in rows)) if rows else len(h) for h in head]
    print("  ".join(h.rjust(w) for h, w in zip(head, widths)))
    for r in rows:
        print("  ".join(_fmt(r[h]).rjust(w) for h, w in zip(head, widths)))
    if args.tsv:
        pathlib.Path(args.tsv).write_text(
            "\n".join(["\t".join(head)] + ["\t".join(_fmt(r[h]) for h in head) for r in rows]) + "\n",
            encoding="utf-8")
    if ident.worst > IDENTITY_TOLERANCE:
        print(f"a game's losses do not telescope: {ident.worst:.1e} > {IDENTITY_TOLERANCE:.0e}")
        return 1
    return 0


def _fmt(value) -> str:
    if isinstance(value, float):
        return f"{value:.3f}"
    return str(value)


# --- research --------------------------------------------------------------

def verdict(threshold: float, fresh: float, losses: dict) -> str:
    """Where an error goes when the engine searches again."""
    if fresh < threshold:
        return "unconfirmed"
    for k in sorted(losses):
        if losses[k] < threshold:
            return "not_reproduced" if k == 1 else f"clock_{k}x"
    return "persists"


def _research_init(exe, engine_hash, judge, judge_hash, judge_nodes) -> None:
    _ignore_interrupts()
    _WORKER["engine"] = Uci(exe, [("Threads", "1"), ("Hash", str(engine_hash))])
    _WORKER["judge"] = Uci(judge, judge_options(judge_hash))
    _WORKER["nodes"] = judge_nodes


def _research_one(item):
    record, game, multiples, threshold = item
    engine, judge, nodes = _WORKER["engine"], _WORKER["judge"], _WORKER["nodes"]
    history = [p.uci for p in game.plies[:record["ply"]]]
    game_nodes = game.plies[record["ply"]].nodes
    found = {}
    for k in multiples:
        engine.new_game()
        found[k] = engine.search(game.start_fen, history, k * game_nodes, False)
    judge.new_game()
    before = judge.search(game.start_fen, history, nodes, True)
    after = {}
    board = chess.Board(game.start_fen)
    for move in history:
        board.push_uci(move)
    for move in dict.fromkeys([record["move"]] + [found[k].best for k in multiples]):
        board.push_uci(move)
        final = terminal(board)
        board.pop()
        value = final or judge.search(game.start_fen, history + [move], nodes, True)
        after[move] = 1.0 - value.es
    fresh = before.es - after[record["move"]]
    losses = {k: before.es - after[found[k].best] for k in multiples}
    out = {key: record[key] for key in ("run", "game", "ply", "name", "opponent", "men",
                                        "clock_frac", "cp_before", "move", "loss_es")}
    out.update({"game_n": game_nodes, "fresh_loss": round(fresh, 4),
                "judge_cp": before.cp, "verdict": verdict(threshold, fresh, losses)})
    for k in multiples:
        out[f"move_{k}x"] = found[k].best
        out[f"loss_{k}x"] = round(losses[k], 4)
        out[f"cp_{k}x"] = found[k].cp
    return out


def cmd_research(args) -> int:
    judged = read_judged(args.judged)
    sample = read_sample(judged.header["sample"].rsplit(" sha256 ", 1)[0])
    if f"sha256 {sample.sha}" not in judged.header["sample"]:
        raise SystemExit("the sample changed since it was judged")
    judge_nodes = int(judged.header["nodes"].split()[0])
    judge_hash = int(judged.header["nodes"].split()[2])
    if sha256(args.judge) not in judged.header["judge"]:
        raise SystemExit("the judge is not the one the moves were judged with")
    games = {(g.run, g.number): g for g, _ in sample_games(sample)}
    chosen = [r for r in judged.records
              if r["mover"] == "engine" and r["loss_es"] >= args.min_loss
              and isinstance(games[(r["run"], r["game"])].plies[r["ply"]].nodes, int)
              and games[(r["run"], r["game"])].plies[r["ply"]].nodes >= args.min_game_nodes]
    chosen.sort(key=lambda r: (-r["loss_es"], r["run"], r["game"], r["ply"]))
    if args.limit:
        chosen = chosen[:args.limit]
    multiples = tuple(int(k) for k in args.multiples.split(","))
    probe = Uci(args.exe, [("Threads", "1"), ("Hash", str(args.hash))])
    engine_name = probe.name
    probe.close()
    print(f"{len(chosen)} engine moves losing at least {args.min_loss}", flush=True)
    items = [(r, games[(r["run"], r["game"])], multiples, args.min_loss) for r in chosen]
    with multiprocessing.Pool(args.workers, _research_init,
                              (args.exe, args.hash, args.judge, judge_hash, judge_nodes)) as pool:
        try:
            rows = list(results(pool.imap(_research_one, items)))
        except KeyboardInterrupt:
            pool.terminate()
            print("stopped: nothing written; the same command starts again", flush=True)
            return 130
    fields = list(rows[0]) if rows else []
    lines = [RESEARCH_MAGIC,
             f"# judged {pathlib.Path(args.judged).resolve()} sha256 {sha256(args.judged)}",
             f"# exe {pathlib.Path(args.exe).resolve()} sha256 {sha256(args.exe)} id {engine_name}",
             f"# hash {args.hash} threads 1 multiples {args.multiples}",
             f"# min_loss {args.min_loss} min_game_nodes {args.min_game_nodes} limit {args.limit}",
             "\t".join(fields)]
    lines += ["\t".join(str(row[f]) for f in fields) for row in rows]
    pathlib.Path(args.out).write_text("\n".join(lines) + "\n", encoding="utf-8")
    return split_summary(rows)


def split_summary(rows: list[dict]) -> int:
    counts = collections.Counter(r["verdict"] for r in rows)
    confirmed = sum(v for k, v in counts.items() if k != "unconfirmed")
    print(f"{len(rows)} moves: " + ", ".join(f"{k} {v}" for k, v in sorted(counts.items())))
    if confirmed:
        clock = sum(v for k, v in counts.items() if k.startswith("clock_"))
        print(f"of {confirmed} confirmed: clock {clock} ({clock / confirmed:.1%}),"
              f" persists {counts['persists']} ({counts['persists'] / confirmed:.1%}),"
              f" not reproduced {counts['not_reproduced']}")
    stays = [r for r in rows if r["verdict"] == "persists"]
    if stays:
        last = max(int(k[3:-1]) for k in rows[0] if k.startswith("cp_") and k.endswith("x"))
        gap = [int(r[f"cp_{last}x"]) - int(r["judge_cp"]) for r in stays
               if abs(int(r[f"cp_{last}x"])) < MATE_CP - 1000 and abs(int(r["judge_cp"])) < MATE_CP - 1000]
        if gap:
            print(f"persisting errors: the engine's score at {last}x minus the judge's,"
                  f" mean {sum(gap) / len(gap):+.1f} cp over {len(gap)}")
    return 0


def cmd_split(args) -> int:
    lines = pathlib.Path(args.research).read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != RESEARCH_MAGIC:
        raise SystemExit(f"{args.research}: not a research file")
    body = [line for line in lines[1:] if not line.startswith("#")]
    columns = body[0].split("\t")
    return split_summary([dict(zip(columns, line.split("\t"))) for line in body[1:]])


# --- main ------------------------------------------------------------------

def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="command", required=True)

    s = sub.add_parser("sample", help="select the games to judge")
    s.add_argument("--run", action="append", required=True, help="a Colosseum CLI run directory")
    s.add_argument("--engine", required=True, help="the engine's name in the PGN")
    s.add_argument("--spoil-cp", type=int, default=50)
    s.add_argument("--spoil-moves", type=int, default=5)
    s.add_argument("--all", action="store_true", help="every game the engine played")
    s.add_argument("--out", required=True)

    j = sub.add_parser("judge", help="score every move of the sample")
    j.add_argument("sample")
    j.add_argument("--judge", required=True)
    j.add_argument("--nodes", type=int, required=True)
    j.add_argument("--hash", type=int, default=64)
    j.add_argument("--workers", type=int, default=1)
    j.add_argument("--progress", type=int, default=10)
    j.add_argument("--out", required=True)

    t = sub.add_parser("table", help="tabulate the judged moves by cell")
    t.add_argument("judged")
    t.add_argument("--by", default="class,clock",
                   help="comma list of class, clock, men, opponent, reason, name")
    t.add_argument("--reason", action="append", help="keep only these sample reasons")
    t.add_argument("--class-cp", type=int, default=CLASS_CP)
    t.add_argument("--tsv")

    r = sub.add_parser("research", help="search the engine's worst moves again")
    r.add_argument("judged")
    r.add_argument("--exe", required=True, help="the engine binary that played")
    r.add_argument("--judge", required=True)
    r.add_argument("--hash", type=int, default=128, help="the engine's hash, as in the games")
    r.add_argument("--min-loss", type=float, default=0.10)
    r.add_argument("--min-game-nodes", type=int, default=1000)
    r.add_argument("--multiples", default=",".join(str(k) for k in MULTIPLES))
    r.add_argument("--limit", type=int, default=0)
    r.add_argument("--workers", type=int, default=1)
    r.add_argument("--out", required=True)

    p = sub.add_parser("split", help="read a research file's verdicts")
    p.add_argument("research")

    args = ap.parse_args(argv)
    if hasattr(signal, "SIGBREAK"):
        signal.signal(signal.SIGBREAK, signal.default_int_handler)
    return {"sample": cmd_sample, "judge": cmd_judge, "table": cmd_table,
            "research": cmd_research, "split": cmd_split}[args.command](args)


if __name__ == "__main__":
    sys.exit(main())
