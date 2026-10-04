"""Compare SPSA horizons under Rarog's live schedule.

This is a schedule/noise model, not an Elo oracle.  It deliberately models a
separable quadratic because the real search objective is unknown; use it only
to compare horizons and gains while keeping its limitations visible.

Measured inputs:
  * 32 games/iteration and std(W-L) ~= sqrt(32 * 0.55) = 4.2;
  * E[W-L] ~= 0.092 * Elo over a 32-game mini-match;
  * alpha/gamma = 0.601/0.102 and A = iterations/10;
  * gain a is derived exactly as tools/spsa.ps1 derives it from r_end.

A canceled proposal (10k/0.00235, 30 coordinates) is kept
here only as a reproducible lesson in schedule design. It was never launched:
a valid schedule does not establish that a tune has enough expected value.
"""

import math
import random
import statistics

ALPHA, GAMMA = 0.601, 0.102
GAMES, SIG_PER_ELO, GRADIENT_NOISE = 32, 0.092, 4.2


def gain_for(iterations: int, r_end: float) -> tuple[float, float]:
    damping = iterations / 10
    gain = r_end * (damping + iterations) ** ALPHA / iterations ** (2 * GAMMA)
    return damping, gain


def schedule_metrics(iterations: int, r_end: float) -> tuple[float, float]:
    damping, gain = gain_for(iterations, r_end)
    cumulative_learning = 0.0
    flat_variance = 0.0
    for k in range(1, iterations + 1):
        a_t = gain / (k + damping) ** ALPHA
        c_t = k**-GAMMA
        cumulative_learning += a_t
        flat_variance += (a_t / c_t * GRADIENT_NOISE) ** 2
    return cumulative_learning, math.sqrt(flat_variance)


def run_quadratic(
    dimensions: int,
    iterations: int,
    r_end: float,
    loss_per_step: float,
    seed: int,
) -> float:
    rng = random.Random(seed)
    damping, gain = gain_for(iterations, r_end)
    theta = [1.0 if rng.random() < 0.5 else -1.0 for _ in range(dimensions)]
    for k in range(1, iterations + 1):
        a_t = gain / (k + damping) ** ALPHA
        c_t = k**-GAMMA
        delta = [1 if rng.random() < 0.5 else -1 for _ in theta]
        plus = sum((theta[i] + c_t * delta[i]) ** 2 for i in range(dimensions))
        minus = sum((theta[i] - c_t * delta[i]) ** 2 for i in range(dimensions))
        elo_a, elo_b = -loss_per_step * plus, -loss_per_step * minus
        gradient = -SIG_PER_ELO * (elo_a - elo_b) + rng.gauss(0, GRADIENT_NOISE)
        for i in range(dimensions):
            theta[i] -= a_t * gradient / (delta[i] * c_t)
    return math.sqrt(sum(x * x for x in theta) / dimensions)


def main() -> None:
    dimensions = 30
    schedules = [
        ("calibrated-5k", 5000, 0.0031),
        ("canceled-phase4", 10000, 0.00235),
    ]
    print("schedule                 iterations  r_end    sum(a_t)  flat-noise SD (steps)")
    for name, iterations, r_end in schedules:
        learning, flat_sd = schedule_metrics(iterations, r_end)
        print(f"{name:24} {iterations:10}  {r_end:.5f}  {learning:8.3f}  {flat_sd:8.3f}")

    print("\nQuadratic endpoint RMSE in parameter-step units (12 deterministic replicas):")
    for curvature in (0.5, 1.5, 4.0):
        print(f"  curvature {curvature:.1f} Elo/full-step")
        for name, iterations, r_end in schedules:
            values = [
                run_quadratic(
                    dimensions,
                    iterations,
                    r_end,
                    curvature,
                    1000 + replica,
                )
                for replica in range(12)
            ]
            print(f"    {name:22} {statistics.fmean(values):.3f}")


if __name__ == "__main__":
    main()
