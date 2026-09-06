//! Binomial intervals shared by the zone and changeset ledgers.
//! Measurement only: no product verdict depends on an interval.

/// False asks per million events — exact integer arithmetic, floored.
pub fn rate_ppm(k: usize, n: usize) -> u64 {
    if n == 0 {
        return 0;
    }
    (k as u64) * 1_000_000 / (n as u64)
}

/// `P[X <= k]` for `X ~ Bin(n, p)`, by relative mass at the mode.
/// Starting at the largest term avoids underflow in `(1-p)^n` for
/// large counts. Only `+ - * /` on f64, with the same sum order on
/// every platform; the gate recomputes each frozen bound.
fn binom_cdf(k: usize, n: usize, p: f64) -> f64 {
    if k >= n {
        return 1.0;
    }
    let q = 1.0 - p;
    if q <= 0.0 {
        return 0.0;
    }
    let mode = (((n + 1) as f64 * p) as usize).min(n);
    let mut mass = vec![0.0; n + 1];
    mass[mode] = 1.0;
    for i in (1..=mode).rev() {
        mass[i - 1] = mass[i] * i as f64 / (n - i + 1) as f64 * q / p;
    }
    for i in mode..n {
        mass[i + 1] = mass[i] * (n - i) as f64 / (i + 1) as f64 * p / q;
    }
    mass[..=k].iter().sum::<f64>() / mass.iter().sum::<f64>()
}

/// The Clopper–Pearson 95 % UPPER bound on the rate, in parts per
/// million: the largest p whose lower binomial tail still holds
/// 2.5 % of the mass. Bisected on integer ppm over the whole
/// [0, 1e6] range — deterministic, and exact to one ppm.
pub fn cp_upper_ppm(k: usize, n: usize) -> u64 {
    if n == 0 || k >= n {
        return 1_000_000;
    }
    let (mut lo, mut hi) = (0u64, 1_000_000u64);
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if binom_cdf(k, n, mid as f64 / 1_000_000.0) >= 0.025 {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

/// Exact integer per-mille — the rate the ledger states, beside the
/// float percentages the interval needs.
pub fn permille(x: usize, n: usize) -> u64 {
    rate_ppm(x, n) / 1000
}

pub fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

/// The p in [lo, hi] where `cdf(n, x, ·)` — decreasing in p — crosses
/// `target`. 200 halvings: deterministic, and far past f64 precision.
fn solve(n: usize, x: usize, target: f64, lo: f64, hi: f64) -> f64 {
    let (mut lo, mut hi) = (lo, hi);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if binom_cdf(x, n, mid) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Clopper-Pearson two-sided 95 %, as percentages: the upper bound is
/// the p where P(X <= x) = 0.025, the lower the p where
/// P(X <= x-1) = 0.975 (exactly 0 for x = 0). The gate pins this
/// against six intervals already published in docs/FPR-TOMBSTONE.md,
/// so this ledger's intervals are that ledger's.
pub fn clopper_pearson(n: usize, x: usize) -> (f64, f64) {
    if n == 0 {
        return (0.0, 0.0);
    }
    let frac = x as f64 / n as f64;
    let lower = if x == 0 {
        0.0
    } else {
        solve(n, x - 1, 0.975, 0.0, frac)
    };
    (100.0 * lower, 100.0 * solve(n, x, 0.025, frac, 1.0))
}
