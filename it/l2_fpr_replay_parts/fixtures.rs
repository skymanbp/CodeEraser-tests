//! Text-only history for the relocation/stacking predicate battery.

/// Two top-level functions; `relocate_me` is the one commit B moves,
/// wide enough that its lines clear the cost model's >= 19-alnum
/// anchor floor several times over.
pub const ALPHA_A: &str = "fn compute_total(values: &[i64], limit: i64) -> i64 {
    let mut running_total_value = 0;
    for candidate_value in values {
        if *candidate_value > limit {
            running_total_value += candidate_value * 3;
        }
    }
    running_total_value
}

fn relocate_me(rows: &[i64], ceiling: i64) -> i64 {
    let mut accumulated_row_total = 0;
    for row_value in rows {
        if *row_value < ceiling {
            accumulated_row_total += row_value + 11;
        } else {
            accumulated_row_total -= row_value / 5;
        }
    }
    accumulated_row_total
}
";

pub const BETA_A: &str = "fn describe_beta_module(label: &str) -> String {
    format!(\"beta module described as {label}\")
}
";

/// Commit B: `relocate_me` has left alpha.rs …
pub const ALPHA_B: &str = "fn compute_total(values: &[i64], limit: i64) -> i64 {
    let mut running_total_value = 0;
    for candidate_value in values {
        if *candidate_value > limit {
            running_total_value += candidate_value * 3;
        }
    }
    running_total_value
}
";

/// … and arrived in beta.rs verbatim: a cross-file relocation, no
/// unit key duplicated anywhere, so the stacking arm cannot fire.
pub const BETA_B: &str = "fn describe_beta_module(label: &str) -> String {
    format!(\"beta module described as {label}\")
}

fn relocate_me(rows: &[i64], ceiling: i64) -> i64 {
    let mut accumulated_row_total = 0;
    for row_value in rows {
        if *row_value < ceiling {
            accumulated_row_total += row_value + 11;
        } else {
            accumulated_row_total -= row_value / 5;
        }
    }
    accumulated_row_total
}
";

/// Commit C: a SECOND top-level `compute_total` written beside the
/// first — the after side gains a duplicated unit key it did not have
/// (dup_spans fires), 24 novel lines land inside the new occurrence's
/// span (>= stackingNovelFloor), and nothing is deleted (0 * 10 < 24).
/// The three arms hold together, so the core must answer `stacking`.
pub const ALPHA_C: &str = "fn compute_total(values: &[i64], limit: i64) -> i64 {
    let mut running_total_value = 0;
    for candidate_value in values {
        if *candidate_value > limit {
            running_total_value += candidate_value * 3;
        }
    }
    running_total_value
}

fn compute_total(values: &[i64], limit: i64) -> i64 {
    let mut rewritten_running_total = 0;
    let mut rewritten_skipped_count = 0;
    let mut rewritten_largest_value = 0;
    for candidate_element in values {
        let doubled_candidate_element = candidate_element * 2;
        if doubled_candidate_element > limit {
            rewritten_running_total += doubled_candidate_element;
            if doubled_candidate_element > rewritten_largest_value {
                rewritten_largest_value = doubled_candidate_element;
            }
        } else {
            rewritten_skipped_count += 1;
        }
    }
    let averaged_running_total = rewritten_running_total / 2;
    let adjusted_running_total = averaged_running_total + rewritten_skipped_count;
    let bounded_running_total = adjusted_running_total.min(rewritten_largest_value * 4);
    let reported_running_total = bounded_running_total.max(0);
    let padded_reported_total = reported_running_total + rewritten_skipped_count;
    let scaled_reported_total = padded_reported_total * 3;
    let trimmed_reported_total = scaled_reported_total / 3;
    let settled_reported_total = trimmed_reported_total + rewritten_largest_value;
    let final_reported_total = settled_reported_total - rewritten_skipped_count;
    final_reported_total
}
";
