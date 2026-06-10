// Basilisk Texel eval tuner
// Phase: Step 2.2 / 2.4 - trace verification and staged Adam optimizer.
//
// Dataset format: one position per line, "FEN;target"
//   target: white-perspective score in [0,1]. Game results (1/0.5/0) and
//   Stockfish/WDL fractional targets are both accepted.
//
// Build: cmake -DTEXEL=ON -DUSE_PEXT=ON ... then cmake --build in Release.
// Run:
//   basilisk-texel --verify dataset.csv
//   basilisk-texel --tune material train.csv holdout.csv [out/eval_params.txt]

#include <algorithm>
#include <cassert>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <string>
#include <vector>

#include "../../src/eval.h"
#include "../../src/Board.h"
#include "../../src/bitboard.h"
#include "../../src/attacks.h"
#include "../../src/zobrist.h"

// ---------------------------------------------------------------------------
// Position records
// ---------------------------------------------------------------------------
struct TexelPos {
    Board     board;
    float     result;  // white perspective, [0,1]
    int       score;   // evaluate() result from side-to-move perspective
    EvalTrace trace;   // collected after evaluate()
};

struct TuneSet {
    int active_count = 0;
    std::vector<float> result;
    std::vector<float> base_score;
    std::vector<float> coeffs; // row-major: position * active_count + active index

    size_t size() const { return result.size(); }

    const float* row(size_t i) const {
        return coeffs.data() + i * static_cast<size_t>(active_count);
    }
};

struct TuneOptions {
    std::string group;
    std::string train_path;
    std::string holdout_path;
    std::string out_path = "tools/texel/out/eval_params.txt";
    int max_positions = 5'000'000;
    int epochs = 200;
    double lr = 0.3;
};

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------
static bool parse_target(const std::string& text, float& out) {
    if (text == "1-0") {
        out = 1.0f;
        return true;
    }
    if (text == "0-1") {
        out = 0.0f;
        return true;
    }
    if (text == "1/2-1/2") {
        out = 0.5f;
        return true;
    }

    char* end = nullptr;
    float parsed = std::strtof(text.c_str(), &end);
    if (end == text.c_str() || *end != '\0' || parsed < 0.0f || parsed > 1.0f)
        return false;

    out = parsed;
    return true;
}

static bool is_option(const char* s) {
    return s != nullptr && s[0] == '-' && s[1] == '-';
}

static int parse_int_arg(const char* value, const char* name) {
    char* end = nullptr;
    long parsed = std::strtol(value, &end, 10);
    if (end == value || *end != '\0' || parsed < 0 || parsed > 100'000'000L) {
        std::cerr << "Bad value for " << name << ": " << value << "\n";
        std::exit(1);
    }
    return static_cast<int>(parsed);
}

static double parse_double_arg(const char* value, const char* name) {
    char* end = nullptr;
    double parsed = std::strtod(value, &end);
    if (end == value || *end != '\0' || !std::isfinite(parsed) || parsed <= 0.0) {
        std::cerr << "Bad value for " << name << ": " << value << "\n";
        std::exit(1);
    }
    return parsed;
}

static void usage(const char* exe) {
    std::cerr
        << "Usage:\n"
        << "  " << exe << " --verify <dataset.csv>\n"
        << "  " << exe << " --tune <group> <train.csv> <holdout.csv> [out/eval_params.txt] [options]\n"
        << "\n"
        << "Groups:\n"
        << "  material    mg/eg material values for pawn..queen (10 params)\n"
        << "  scalars     non-PST, non-king-safety scalar/table terms\n"
        << "  pawnstruct  doubled/isolated/connected/backward pawn terms\n"
        << "  passers     passed/candidate/supported/free passer terms\n"
        << "  rooks       open/semi/7th/behind-passer rook terms\n"
        << "  minors      bishop pair, knight outpost, trapped bishop terms\n"
        << "  mobility    per-piece safe mobility terms\n"
        << "  threats     pawn threats to minor/rook/queen\n"
        << "  hanging     hanging piece penalties\n"
        << "  misc        passed-king proximity, space, tempo\n"
        << "  kingsafety  king attack, shelter, and storm terms\n"
        << "  pst         PSTs plus material refit\n"
        << "  all         all meaningful eval params\n"
        << "\n"
        << "Options:\n"
        << "  --epochs N         optimizer epochs (default 200)\n"
        << "  --lr X             Adam learning rate (default 0.3)\n"
        << "  --max-positions N  cap positions loaded from each file (default 5000000; 0 = all)\n";
}

static TuneOptions parse_tune_options(int argc, char* argv[]) {
    if (argc < 5) {
        usage(argv[0]);
        std::exit(1);
    }

    TuneOptions opts;
    opts.group = argv[2];
    opts.train_path = argv[3];
    opts.holdout_path = argv[4];

    int argi = 5;
    if (argi < argc && !is_option(argv[argi])) {
        opts.out_path = argv[argi++];
    }

    while (argi < argc) {
        std::string opt = argv[argi++];
        if (opt == "--epochs" && argi < argc) {
            opts.epochs = parse_int_arg(argv[argi++], "--epochs");
        } else if (opt == "--lr" && argi < argc) {
            opts.lr = parse_double_arg(argv[argi++], "--lr");
        } else if (opt == "--max-positions" && argi < argc) {
            opts.max_positions = parse_int_arg(argv[argi++], "--max-positions");
        } else {
            std::cerr << "Unknown or incomplete option: " << opt << "\n";
            usage(argv[0]);
            std::exit(1);
        }
    }

    if (opts.epochs <= 0) {
        std::cerr << "--epochs must be positive.\n";
        std::exit(1);
    }

    return opts;
}

// ---------------------------------------------------------------------------
// Flat parameter helpers (matches EVAL_PARAM_LIST order)
// ---------------------------------------------------------------------------
static void read_weights(const EvalParams& p, double* w) {
    int idx = 0;
#define X(name, member, len) \
    { const int* ptr = eval_param_cptr(p.member); \
      for (int i = 0; i < (len); i++) w[idx++] = static_cast<double>(ptr[i]); }
    EVAL_PARAM_LIST(X)
#undef X
}

static void write_weights(EvalParams& p, const double* w) {
    int idx = 0;
#define X(name, member, len) \
    { int* ptr = eval_param_ptr(p.member); \
      for (int i = 0; i < (len); i++) ptr[i] = static_cast<int>(std::round(w[idx++])); }
    EVAL_PARAM_LIST(X)
#undef X
}

static void append_group(std::vector<int>& out, int group) {
    int base = eval_param_offset(group);
    int len = EVAL_PARAM_LENS[group];
    for (int i = 0; i < len; ++i)
        out.push_back(base + i);
}

static void append_groups(std::vector<int>& out, int first, int last) {
    for (int g = first; g <= last; ++g)
        append_group(out, g);
}

static void append_material(std::vector<int>& out) {
    int mg = eval_param_offset(EPG_MgVal);
    int eg = eval_param_offset(EPG_EgVal);
    for (int pt = PAWN; pt <= QUEEN; ++pt) {
        out.push_back(mg + pt);
        out.push_back(eg + pt);
    }
}

static void unique_sort(std::vector<int>& xs) {
    std::sort(xs.begin(), xs.end());
    xs.erase(std::unique(xs.begin(), xs.end()), xs.end());
}

static std::vector<int> active_indices_for_group(const std::string& group) {
    std::vector<int> active;

    if (group == "material") {
        append_material(active);
    } else if (group == "scalars") {
        for (int g = EPG_PassedMg; g <= EPG_Tempo; ++g) {
            if (g >= EPG_KsUnit && g <= EPG_StormWeightAdj)
                continue;
            append_group(active, g);
        }
    } else if (group == "pawnstruct" || group == "pawns") {
        append_groups(active, EPG_DoubledMg, EPG_BackwardEg);
    } else if (group == "passers") {
        append_groups(active, EPG_PassedMg, EPG_PassSafeEg);
        append_group(active, EPG_ProxBase);
    } else if (group == "rooks") {
        append_groups(active, EPG_RookOpenMg, EPG_EnemyRookPasserEg);
    } else if (group == "minors") {
        append_group(active, EPG_BpMg);
        append_group(active, EPG_BpEg);
        append_group(active, EPG_KnightOutpostMg);
        append_group(active, EPG_KnightOutpostEg);
        append_group(active, EPG_TrappedMg);
        append_group(active, EPG_TrappedEg);
    } else if (group == "mobility") {
        append_group(active, EPG_MobMg);
        append_group(active, EPG_MobEg);
    } else if (group == "threats") {
        append_groups(active, EPG_ThreatMinorMg, EPG_ThreatQueenEg);
    } else if (group == "hanging") {
        append_group(active, EPG_HangPen);
    } else if (group == "misc") {
        append_group(active, EPG_ProxBase);
        append_group(active, EPG_SpaceMg);
        append_group(active, EPG_Tempo);
    } else if (group == "kingsafety" || group == "king") {
        for (int g = EPG_KsUnit; g <= EPG_StormWeightAdj; ++g)
            append_group(active, g);
    } else if (group == "pst") {
        append_material(active);
        for (int g = EPG_PstMgPawn; g <= EPG_PstEgKing; ++g)
            append_group(active, g);
    } else if (group == "all") {
        append_material(active);
        for (int g = EPG_PstMgPawn; g <= EPG_PstEgKing; ++g)
            append_group(active, g);
        for (int g = EPG_PassedMg; g <= EPG_Tempo; ++g)
            append_group(active, g);
    } else {
        std::cerr << "Unknown tune group '" << group << "'.\n";
        std::exit(1);
    }

    unique_sort(active);
    if (active.empty()) {
        std::cerr << "Tune group '" << group << "' has no active params.\n";
        std::exit(1);
    }
    return active;
}

static float trace_coeff(const EvalTrace& tr, int idx) {
    return static_cast<float>(
        tr.mg[idx] * tr.phase + tr.eg[idx] * (24 - tr.phase)) / 24.0f;
}

static int default_score_white(const EvalTrace& tr) {
    return reconstruct(tr, g_eval_params) + tr.rest;
}

static int flat_index(int group, int idx = 0) {
    return eval_param_offset(group) + idx;
}

static float linear_delta_scale(const Board& b) {
    float scale = 1.0f;

    constexpr Bitboard DARK_SQ = 0x55AA55AA55AA55AAULL;
    bool wb1 = !more_than_one(b.pieces[WHITE][BISHOP]) && b.pieces[WHITE][BISHOP];
    bool bb1 = !more_than_one(b.pieces[BLACK][BISHOP]) && b.pieces[BLACK][BISHOP];
    if (wb1 && bb1) {
        bool wb_dark = (b.pieces[WHITE][BISHOP] & DARK_SQ) != 0;
        bool bb_dark = (b.pieces[BLACK][BISHOP] & DARK_SQ) != 0;
        if (wb_dark != bb_dark) {
            int total_pawns = popcount(b.pieces[WHITE][PAWN] | b.pieces[BLACK][PAWN]);
            scale *= static_cast<float>(32 + total_pawns * 4) / 48.0f;
        }
    }

    auto only_king = [&](Color c) {
        return b.occupancy[c] == sq_bb(b.king_sq[c]);
    };
    auto only_knights = [&](Color c, int n) {
        return !b.pieces[c][PAWN] && !b.pieces[c][BISHOP]
            && !b.pieces[c][ROOK] && !b.pieces[c][QUEEN]
            && popcount(b.pieces[c][KNIGHT]) == n;
    };
    if ((only_king(WHITE) && only_knights(BLACK, 2)) ||
        (only_king(BLACK) && only_knights(WHITE, 2))) {
        return 0.0f;
    }

    if (b.halfmove_clock > 0)
        scale *= static_cast<float>(std::max(0, 100 - b.halfmove_clock)) / 100.0f;

    return scale;
}

static void clamp_range(double* w, int idx, double lo, double hi) {
    w[idx] = std::clamp(w[idx], lo, hi);
}

static void clamp_group_range(double* w, int group, double lo, double hi) {
    int base = eval_param_offset(group);
    int len = EVAL_PARAM_LENS[group];
    for (int i = 0; i < len; ++i)
        clamp_range(w, base + i, lo, hi);
}

static void enforce_non_decreasing(double* w, int group, int first, int last) {
    int base = eval_param_offset(group);
    for (int i = first + 1; i <= last; ++i)
        w[base + i] = std::max(w[base + i], w[base + i - 1]);
}

static void clamp_weights_for_group(const std::string&, double* w) {
    int mg = eval_param_offset(EPG_MgVal);
    int eg = eval_param_offset(EPG_EgVal);

    w[mg + NO_PIECE_TYPE] = 0.0;
    w[eg + NO_PIECE_TYPE] = 0.0;
    w[mg + KING] = 0.0;
    w[eg + KING] = 0.0;

    for (int pt = PAWN; pt <= QUEEN; ++pt) {
        w[mg + pt] = std::clamp(w[mg + pt], 1.0, 2000.0);
        w[eg + pt] = std::clamp(w[eg + pt], 1.0, 2000.0);
    }

    // Sign/shape clamps for scalar subgroups. These are deliberately simple:
    // they prevent obviously nonsensical candidates while leaving SPRT to
    // decide whether a plausible fit transfers to strength.
    clamp_group_range(w, EPG_DoubledMg, -200.0, 0.0);
    clamp_group_range(w, EPG_DoubledEg, -200.0, 0.0);
    clamp_group_range(w, EPG_IsolatedMg, -200.0, 0.0);
    clamp_group_range(w, EPG_IsolatedEg, -200.0, 0.0);
    clamp_group_range(w, EPG_BackwardMg, -200.0, 0.0);
    clamp_group_range(w, EPG_BackwardEg, -200.0, 0.0);
    clamp_group_range(w, EPG_ConnectedMg, 0.0, 200.0);
    clamp_group_range(w, EPG_ConnectedEg, 0.0, 200.0);

    for (int r = 0; r < 8; ++r) {
        clamp_range(w, flat_index(EPG_PassedMg, r), r == 0 || r == 7 ? 0.0 : 0.0, r == 0 || r == 7 ? 0.0 : 400.0);
        clamp_range(w, flat_index(EPG_PassedEg, r), r == 0 || r == 7 ? 0.0 : 0.0, r == 0 || r == 7 ? 0.0 : 400.0);
    }
    enforce_non_decreasing(w, EPG_PassedMg, 1, 6);
    enforce_non_decreasing(w, EPG_PassedEg, 1, 6);
    clamp_group_range(w, EPG_PassSuppMg, 0.0, 200.0);
    clamp_group_range(w, EPG_PassSuppEgBase, 0.0, 200.0);
    clamp_group_range(w, EPG_PassSuppEgRank, 0.0, 50.0);
    clamp_group_range(w, EPG_CandMg, 0.0, 200.0);
    clamp_group_range(w, EPG_CandEg, 0.0, 200.0);
    clamp_group_range(w, EPG_PassFreeMg, 0.0, 100.0);
    clamp_group_range(w, EPG_PassFreeEg, 0.0, 100.0);
    clamp_group_range(w, EPG_PassSafeEg, 0.0, 100.0);
    clamp_group_range(w, EPG_ProxBase, 0.0, 50.0);

    clamp_group_range(w, EPG_BpMg, 0.0, 200.0);
    clamp_group_range(w, EPG_BpEg, 0.0, 200.0);
    clamp_group_range(w, EPG_KnightOutpostMg, 0.0, 200.0);
    clamp_group_range(w, EPG_KnightOutpostEg, 0.0, 200.0);
    clamp_group_range(w, EPG_TrappedMg, 0.0, 200.0);
    clamp_group_range(w, EPG_TrappedEg, 0.0, 200.0);

    clamp_group_range(w, EPG_RookOpenMg, 0.0, 200.0);
    clamp_group_range(w, EPG_RookOpenEg, 0.0, 200.0);
    clamp_group_range(w, EPG_RookSemiMg, 0.0, 200.0);
    clamp_group_range(w, EPG_RookSemiEg, 0.0, 200.0);
    clamp_group_range(w, EPG_Rook7thMg, 0.0, 200.0);
    clamp_group_range(w, EPG_Rook7thEg, 0.0, 200.0);
    clamp_group_range(w, EPG_RookBehindPasserMg, 0.0, 200.0);
    clamp_group_range(w, EPG_RookBehindPasserEg, 0.0, 200.0);
    clamp_group_range(w, EPG_EnemyRookPasserMg, 0.0, 200.0);
    clamp_group_range(w, EPG_EnemyRookPasserEg, 0.0, 200.0);

    for (int pt = 0; pt < PIECE_TYPE_NB; ++pt) {
        double hi = (pt >= KNIGHT && pt <= QUEEN) ? 50.0 : 0.0;
        clamp_range(w, flat_index(EPG_MobMg, pt), 0.0, hi);
        clamp_range(w, flat_index(EPG_MobEg, pt), 0.0, hi);

        hi = (pt >= KNIGHT && pt <= QUEEN) ? 200.0 : 0.0;
        clamp_range(w, flat_index(EPG_HangPen, pt), 0.0, hi);
    }

    clamp_group_range(w, EPG_ThreatMinorMg, 0.0, 200.0);
    clamp_group_range(w, EPG_ThreatMinorEg, 0.0, 200.0);
    clamp_group_range(w, EPG_ThreatRookMg, 0.0, 200.0);
    clamp_group_range(w, EPG_ThreatRookEg, 0.0, 200.0);
    clamp_group_range(w, EPG_ThreatQueenMg, 0.0, 200.0);
    clamp_group_range(w, EPG_ThreatQueenEg, 0.0, 200.0);
    w[flat_index(EPG_ThreatRookMg)] = std::max(w[flat_index(EPG_ThreatRookMg)], w[flat_index(EPG_ThreatMinorMg)]);
    w[flat_index(EPG_ThreatQueenMg)] = std::max(w[flat_index(EPG_ThreatQueenMg)], w[flat_index(EPG_ThreatRookMg)]);
    w[flat_index(EPG_ThreatRookEg)] = std::max(w[flat_index(EPG_ThreatRookEg)], w[flat_index(EPG_ThreatMinorEg)]);
    w[flat_index(EPG_ThreatQueenEg)] = std::max(w[flat_index(EPG_ThreatQueenEg)], w[flat_index(EPG_ThreatRookEg)]);

    clamp_group_range(w, EPG_SpaceMg, 0.0, 50.0);
    clamp_group_range(w, EPG_Tempo, 0.0, 50.0);
}

// ---------------------------------------------------------------------------
// Dataset loading
// ---------------------------------------------------------------------------
static std::vector<TexelPos> load_verify_dataset(const std::string& path,
                                                 int max_positions = 0) {
    std::ifstream f(path);
    if (!f) {
        std::cerr << "Cannot open dataset: " << path << "\n";
        std::exit(1);
    }

    std::vector<TexelPos> out;
    std::string line;
    int lineno = 0;
    auto evaluator_ptr = std::make_unique<Evaluator>();
    Evaluator& evaluator = *evaluator_ptr;

    while (std::getline(f, line)) {
        ++lineno;
        if (line.empty() || line[0] == '#')
            continue;

        auto sep = line.rfind(';');
        if (sep == std::string::npos) {
            std::cerr << "Bad line " << lineno << " (no ';'): " << line << "\n";
            continue;
        }

        std::string fen = line.substr(0, sep);
        std::string target = line.substr(sep + 1);

        float result = 0.5f;
        if (!parse_target(target, result)) {
            std::cerr << "Unknown result '" << target << "' at line " << lineno << "\n";
            continue;
        }

        TexelPos tp;
        tp.result = result;
        std::string err;
        if (!tp.board.try_set_fen(fen, &err)) {
            std::cerr << "FEN error at line " << lineno << ": " << err << "\n";
            continue;
        }

        g_trace = {};
        tp.score = evaluator.evaluate(tp.board);
        tp.trace = g_trace;

        int score_white = (tp.board.side_to_move == WHITE) ? tp.score : -tp.score;
        tp.trace.rest = score_white - reconstruct(tp.trace, g_eval_params);

        out.push_back(std::move(tp));

        if (max_positions > 0 && static_cast<int>(out.size()) >= max_positions)
            break;
    }

    return out;
}

static TuneSet load_tune_dataset(const std::string& path,
                                 const std::vector<int>& active,
                                 int max_positions) {
    std::ifstream f(path);
    if (!f) {
        std::cerr << "Cannot open dataset: " << path << "\n";
        std::exit(1);
    }

    TuneSet out;
    out.active_count = static_cast<int>(active.size());

    if (max_positions > 0) {
        out.result.reserve(static_cast<size_t>(max_positions));
        out.base_score.reserve(static_cast<size_t>(max_positions));
        const size_t coeff_reserve = static_cast<size_t>(max_positions) * active.size();
        if (coeff_reserve <= 100'000'000ULL)
            out.coeffs.reserve(coeff_reserve);
    }

    std::string line;
    int lineno = 0;
    int skipped = 0;
    Board board;
    auto evaluator_ptr = std::make_unique<Evaluator>();
    Evaluator& evaluator = *evaluator_ptr;

    while (std::getline(f, line)) {
        ++lineno;
        if (line.empty() || line[0] == '#')
            continue;

        auto sep = line.rfind(';');
        if (sep == std::string::npos) {
            ++skipped;
            if (skipped <= 5)
                std::cerr << "Bad line " << lineno << " (no ';'): " << line << "\n";
            continue;
        }

        std::string fen = line.substr(0, sep);
        std::string target = line.substr(sep + 1);

        float result = 0.5f;
        if (!parse_target(target, result)) {
            ++skipped;
            if (skipped <= 5)
                std::cerr << "Unknown result '" << target << "' at line " << lineno << "\n";
            continue;
        }

        std::string err;
        if (!board.try_set_fen(fen, &err)) {
            ++skipped;
            if (skipped <= 5)
                std::cerr << "FEN error at line " << lineno << ": " << err << "\n";
            continue;
        }

        g_trace = {};
        int score = evaluator.evaluate(board);
        int score_white = (board.side_to_move == WHITE) ? score : -score;
        g_trace.rest = score_white - reconstruct(g_trace, g_eval_params);

        out.result.push_back(result);
        out.base_score.push_back(static_cast<float>(default_score_white(g_trace)));
        float delta_scale = linear_delta_scale(board);
        for (int idx : active)
            out.coeffs.push_back(trace_coeff(g_trace, idx) * delta_scale);

        if (max_positions > 0 && static_cast<int>(out.size()) >= max_positions)
            break;
    }

    if (out.size() == 0) {
        std::cerr << "No positions loaded from " << path << ".\n";
        std::exit(1);
    }

    if (skipped > 0) {
        std::cerr << "Skipped " << skipped << " malformed rows from " << path << ".\n";
    }

    return out;
}

// ---------------------------------------------------------------------------
// Reconstruction check
// ---------------------------------------------------------------------------
static void cmd_verify(const std::string& path) {
    constexpr int VERIFY_COUNT = 10000;
    std::cout << "Loading up to " << VERIFY_COUNT
              << " positions from " << path << " ...\n";

    auto positions = load_verify_dataset(path, VERIFY_COUNT);
    if (positions.empty()) {
        std::cerr << "No positions loaded.\n";
        std::exit(1);
    }

    std::cout << "Loaded " << positions.size() << " positions. Verifying reconstruction...\n";

    auto ev_ptr = std::make_unique<Evaluator>();
    Evaluator& evaluator = *ev_ptr;
    int mismatches = 0;
    int max_err = 0;

    for (auto& tp : positions) {
        int fresh = evaluator.evaluate(tp.board);
        int fresh_white = (tp.board.side_to_move == WHITE) ? fresh : -fresh;
        int recon = reconstruct(tp.trace, g_eval_params) + tp.trace.rest;

        int err = std::abs(fresh_white - recon);
        if (err != 0) {
            ++mismatches;
            max_err = std::max(max_err, err);
            if (mismatches <= 5) {
                std::cerr << "MISMATCH: actual=" << fresh_white
                          << " recon=" << recon
                          << " diff=" << err
                          << " fen=" << tp.board.get_fen() << "\n";
            }
        }
    }

    if (mismatches == 0) {
        std::cout << "PASS: all " << positions.size()
                  << " positions reconstruct exactly.\n";
    } else {
        std::cout << "FAIL: " << mismatches << " / " << positions.size()
                  << " positions differ (max error = " << max_err << ").\n";
        std::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Sigmoid / loss helpers
// ---------------------------------------------------------------------------
static double sigmoid(double score, double K) {
    return 1.0 / (1.0 + std::exp(-K * score / 400.0));
}

static double default_loss(const TuneSet& set, double K) {
    double loss = 0.0;
    for (size_t i = 0; i < set.size(); ++i) {
        double sig = sigmoid(static_cast<double>(set.base_score[i]), K);
        double diff = static_cast<double>(set.result[i]) - sig;
        loss += diff * diff;
    }
    return loss / static_cast<double>(set.size());
}

static double score_from_weights(const TuneSet& set,
                                 size_t pos,
                                 const std::vector<int>& active,
                                 const double* base_w,
                                 const double* w) {
    double score = static_cast<double>(set.base_score[pos]);
    const float* coeff = set.row(pos);
    for (size_t j = 0; j < active.size(); ++j) {
        int idx = active[j];
        score += static_cast<double>(coeff[j]) * (w[idx] - base_w[idx]);
    }
    return score;
}

static double traced_loss(const TuneSet& set,
                          const std::vector<int>& active,
                          const double* base_w,
                          const double* w,
                          double K) {
    double loss = 0.0;
    for (size_t i = 0; i < set.size(); ++i) {
        double sig = sigmoid(score_from_weights(set, i, active, base_w, w), K);
        double diff = static_cast<double>(set.result[i]) - sig;
        loss += diff * diff;
    }
    return loss / static_cast<double>(set.size());
}

// ---------------------------------------------------------------------------
// K calibration (golden-section search on validation set)
// ---------------------------------------------------------------------------
static double fit_K(const TuneSet& positions) {
    std::cout << "Fitting K on " << positions.size() << " positions... ";
    double lo = 0.5;
    double hi = 2.5;
    for (int i = 0; i < 50; i++) {
        double m1 = lo + (hi - lo) / 3.0;
        double m2 = hi - (hi - lo) / 3.0;
        if (default_loss(positions, m1) < default_loss(positions, m2))
            hi = m2;
        else
            lo = m1;
    }
    double K = (lo + hi) / 2.0;
    std::cout << "K = " << K << "\n";
    return K;
}

// ---------------------------------------------------------------------------
// Output helpers
// ---------------------------------------------------------------------------
static void write_eval_params_file(const std::string& out_path) {
    std::filesystem::path path(out_path);
    if (!path.parent_path().empty())
        std::filesystem::create_directories(path.parent_path());

    std::ofstream out(out_path);
    if (!out) {
        std::cerr << "Cannot write " << out_path << "\n";
        std::exit(1);
    }

#define X(name, member, len) \
    { const int* ptr = eval_param_cptr(g_eval_params.member); \
      for (int i = 0; i < (len); i++) out << #name << " " << i << " " << ptr[i] << "\n"; }
    EVAL_PARAM_LIST(X)
#undef X
}

static void print_material_delta(const double* base_w, const double* w) {
    static const char* names[] = {"-", "Pawn", "Knight", "Bishop", "Rook", "Queen", "King"};
    int mg = eval_param_offset(EPG_MgVal);
    int eg = eval_param_offset(EPG_EgVal);

    std::cout << "\nMaterial values (rounded output):\n";
    std::cout << "Piece     MG old -> new    EG old -> new\n";
    for (int pt = PAWN; pt <= QUEEN; ++pt) {
        int mg_old = static_cast<int>(std::round(base_w[mg + pt]));
        int mg_new = static_cast<int>(std::round(w[mg + pt]));
        int eg_old = static_cast<int>(std::round(base_w[eg + pt]));
        int eg_new = static_cast<int>(std::round(w[eg + pt]));
        std::printf("%-8s %5d -> %-5d  %5d -> %-5d\n",
                    names[pt], mg_old, mg_new, eg_old, eg_new);
    }
}

struct FlatParamInfo {
    const char* name;
    int index;
};

static std::vector<FlatParamInfo> flat_param_info() {
    std::vector<FlatParamInfo> info;
    info.reserve(EVAL_PARAM_FLAT_SIZE);
#define X(name, member, len) \
    for (int i = 0; i < (len); ++i) info.push_back({#name, i});
    EVAL_PARAM_LIST(X)
#undef X
    return info;
}

static void print_active_deltas(const std::vector<int>& active,
                                const double* base_w,
                                const double* w) {
    static const std::vector<FlatParamInfo> info = flat_param_info();

    int changed = 0;
    for (int idx : active) {
        int old_v = static_cast<int>(std::round(base_w[idx]));
        int new_v = static_cast<int>(std::round(w[idx]));
        if (old_v != new_v)
            ++changed;
    }

    std::cout << "\nActive parameter deltas (rounded output): "
              << changed << " changed / " << active.size() << " active\n";
    if (changed == 0)
        return;

    std::cout << "Param                  Idx  Old -> New   Delta\n";
    int printed = 0;
    for (int idx : active) {
        int old_v = static_cast<int>(std::round(base_w[idx]));
        int new_v = static_cast<int>(std::round(w[idx]));
        if (old_v == new_v)
            continue;
        const FlatParamInfo& param = info[static_cast<size_t>(idx)];
        std::printf("%-22s %3d  %4d -> %-4d  %+d\n",
                    param.name, param.index, old_v, new_v, new_v - old_v);
        if (++printed >= 120 && changed > printed) {
            std::cout << "... " << (changed - printed)
                      << " additional changed params omitted\n";
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Adam optimizer using active trace coefficients
// ---------------------------------------------------------------------------
static void cmd_tune(const TuneOptions& opts) {
    constexpr double BETA1 = 0.9;
    constexpr double BETA2 = 0.999;
    constexpr double EPS = 1e-8;

    std::vector<int> active = active_indices_for_group(opts.group);

    std::cout << "Tune group: " << opts.group
              << " (" << active.size() << " active params)\n";

    std::cout << "Loading train dataset from " << opts.train_path << " ...\n";
    TuneSet train = load_tune_dataset(opts.train_path, active, opts.max_positions);
    std::cout << "Loaded " << train.size() << " train positions.\n";

    std::cout << "Loading holdout dataset from " << opts.holdout_path << " ...\n";
    TuneSet holdout = load_tune_dataset(opts.holdout_path, active, opts.max_positions);
    std::cout << "Loaded " << holdout.size() << " holdout positions.\n";

    double K = fit_K(holdout);

    double base_w[EVAL_PARAM_FLAT_SIZE];
    double w[EVAL_PARAM_FLAT_SIZE];
    read_weights(g_eval_params, base_w);
    std::copy(base_w, base_w + EVAL_PARAM_FLAT_SIZE, w);

    double initial_train_loss = traced_loss(train, active, base_w, w, K);
    double initial_holdout_loss = traced_loss(holdout, active, base_w, w, K);
    std::cout << "Initial train loss  = " << initial_train_loss << "\n";
    std::cout << "Initial holdout loss= " << initial_holdout_loss << "\n";

    double best_w[EVAL_PARAM_FLAT_SIZE];
    std::copy(w, w + EVAL_PARAM_FLAT_SIZE, best_w);
    double best_holdout = initial_holdout_loss;
    int best_epoch = 0;

    std::vector<double> grad(active.size(), 0.0);
    std::vector<double> m(active.size(), 0.0);
    std::vector<double> v(active.size(), 0.0);
    int t = 0;
    const double n = static_cast<double>(train.size());

    for (int epoch = 1; epoch <= opts.epochs; epoch++) {
        auto t0 = std::chrono::steady_clock::now();
        std::fill(grad.begin(), grad.end(), 0.0);

        for (size_t i = 0; i < train.size(); ++i) {
            double score = score_from_weights(train, i, active, base_w, w);
            double sig = sigmoid(score, K);
            double err = static_cast<double>(train.result[i]) - sig;
            double dsig = sig * (1.0 - sig);
            double coeff = -2.0 * err * dsig * (K / 400.0);

            const float* row = train.row(i);
            for (size_t j = 0; j < active.size(); ++j)
                grad[j] += coeff * static_cast<double>(row[j]);
        }

        ++t;
        double bc1 = 1.0 - std::pow(BETA1, static_cast<double>(t));
        double bc2 = 1.0 - std::pow(BETA2, static_cast<double>(t));

        for (size_t j = 0; j < active.size(); ++j) {
            double g = grad[j] / n;
            m[j] = BETA1 * m[j] + (1.0 - BETA1) * g;
            v[j] = BETA2 * v[j] + (1.0 - BETA2) * g * g;
            double m_hat = m[j] / bc1;
            double v_hat = v[j] / bc2;
            w[active[j]] -= opts.lr * m_hat / (std::sqrt(v_hat) + EPS);
        }

        clamp_weights_for_group(opts.group, w);

        auto t1 = std::chrono::steady_clock::now();
        double ms = std::chrono::duration<double, std::milli>(t1 - t0).count();
        double holdout_loss = traced_loss(holdout, active, base_w, w, K);
        if (holdout_loss < best_holdout) {
            best_holdout = holdout_loss;
            best_epoch = epoch;
            std::copy(w, w + EVAL_PARAM_FLAT_SIZE, best_w);
        }

        if (epoch == 1 || epoch % 10 == 0 || epoch == opts.epochs) {
            double train_loss = traced_loss(train, active, base_w, w, K);
            std::printf("Epoch %4d  train=%.8f  holdout=%.8f  %.0f ms\n",
                        epoch, train_loss, holdout_loss, ms);
        }
    }

    if (best_epoch != opts.epochs) {
        std::copy(best_w, best_w + EVAL_PARAM_FLAT_SIZE, w);
        std::cout << "Restored best holdout epoch " << best_epoch
                  << " (holdout=" << best_holdout << ").\n";
    } else {
        std::cout << "Best holdout was final epoch " << best_epoch
                  << " (holdout=" << best_holdout << ").\n";
    }

    write_weights(g_eval_params, w);
    print_material_delta(base_w, w);
    print_active_deltas(active, base_w, w);
    write_eval_params_file(opts.out_path);
    std::cout << "Tuned weights written to " << opts.out_path << "\n";
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------
int main(int argc, char* argv[]) {
    init_bitboards();
    init_attacks();
    Zobrist::init();
    init_eval_tables(g_eval_params);

    if (argc < 2) {
        usage(argv[0]);
        return 1;
    }

    std::string mode = argv[1];

    if (mode == "--verify") {
        if (argc != 3) {
            usage(argv[0]);
            return 1;
        }
        cmd_verify(argv[2]);
    } else if (mode == "--tune") {
        TuneOptions opts = parse_tune_options(argc, argv);
        cmd_tune(opts);
    } else {
        std::cerr << "Unknown mode '" << mode << "'. Use --verify or --tune.\n";
        usage(argv[0]);
        return 1;
    }

    return 0;
}
