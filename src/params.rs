pub(crate) const fn direct_check_bonus() -> i32 {
    32_000
}

pub(crate) const fn probcut_base_margin() -> i32 {
    188
}

pub(crate) const fn probcut_depth_margin() -> i32 {
    4
}

pub(crate) const fn probcut_improving_bonus() -> i32 {
    28
}

pub(crate) const fn singular_exact_base_margin() -> i32 {
    12
}

pub(crate) const fn singular_exact_depth_margin() -> i32 {
    3
}

pub(crate) const fn singular_lower_base_margin() -> i32 {
    24
}

pub(crate) const fn singular_lower_depth_margin() -> i32 {
    4
}

pub(crate) const fn tt_cutoff_history_divisor_base() -> i32 {
    4
}

pub(crate) const fn fail_low_parent_history_divisor() -> i32 {
    10
}

pub(crate) const fn fail_low_parent_eval_divisor() -> i32 {
    16
}

pub(crate) const fn late_move_prune_base() -> i32 {
    4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_params_have_expected_signs_and_ordering() {
        assert!(direct_check_bonus() > 0);
        assert!(probcut_base_margin() > probcut_improving_bonus());
        assert!(singular_lower_base_margin() > singular_exact_base_margin());
        assert!(singular_lower_depth_margin() >= singular_exact_depth_margin());
        assert!(late_move_prune_base() > 0);
    }
}
