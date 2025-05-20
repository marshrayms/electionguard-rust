// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(clippy::expect_used)]
#![deny(clippy::manual_assert)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::assertions_on_constants)]

//=================================================================================================|

/// Decomposes a `i128` into a `u128` absolute value and sign flag.
const fn i128_to_u128_isneg(i: i128) -> (u128, bool) {
    let neg = i.is_negative();
    let mut u = i as u128;
    if neg {
        u = !u;
        u = u.wrapping_add(1)
    }
    (u, neg)
}

/// Converts `u128` to an `f64`. Some precision will be lost unless `u < 2^53`.
pub const fn u128_to_f64(u: u128) -> f64 {
    const U53_MASK: u64 = (1_u64 << 53) - 1;
    const TWO53: f64 = (1_u64 << 53) as f64;
    const TWO106: f64 = TWO53 * TWO53;

    let uh: u32 = (u >> 106) as u32; // 22 bits
    let um: u64 = ((u >> 53) as u64) & U53_MASK;
    let ul: u64 = (u as u64) & U53_MASK;

    (uh as f64) * TWO106 + (um as f64) * TWO53 + (ul as f64)
}

/// Converts `u128` to an `f64`. Some precision will be lost unless `-2^53 < i < 2^53`.
pub const fn i128_to_f64(i: i128) -> f64 {
    let (u, neg) = i128_to_u128_isneg(i);
    let sign = (1_i8 - (neg as i8) * 2) as f64;
    u128_to_f64(u) * sign
}

//=================================================================================================|

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod t {
    use super::*;
    use insta::assert_debug_snapshot;
    use static_assertions::const_assert_eq;

    #[test]
    fn t1_i128_to_u128_isneg() {
        #[allow(dead_code)]
        const C: (u128, bool) = i128_to_u128_isneg(0);
        const_assert_eq!(C.0, 0);
        const_assert_eq!(C.1, false);

        let pr = i128_to_u128_isneg(1);
        assert_debug_snapshot!(pr.0, @"1");
        assert_debug_snapshot!(pr.1, @r#"false"#);

        let pr = i128_to_u128_isneg(-1);
        assert_debug_snapshot!(pr.0, @r#"1"#);
        assert_debug_snapshot!(pr.1, @r#"true"#);

        let pr = i128_to_u128_isneg(i128::MIN);
        assert_debug_snapshot!(pr.0, @"170141183460469231731687303715884105728");
        assert_debug_snapshot!(pr.1, @r#"true"#);

        let pr = i128_to_u128_isneg(i128::MAX);
        assert_debug_snapshot!(pr.0, @"170141183460469231731687303715884105727");
        assert_debug_snapshot!(pr.1, @r#"false"#);
    }

    struct AE {
        actual: f64,
        expected: f64,
    }
    impl AE {
        fn abs_diff(&self) -> f64 {
            (self.actual - self.expected).abs()
        }
        fn pct_err(&self) -> f64 {
            let abs_diff = self.abs_diff();
            let divisor = self.expected.abs().max(1.0e-12);
            abs_diff / divisor * 100.0
        }
        fn pct_err_is_acceptable(&self, print_anyway: bool) -> bool {
            let pct_err = self.pct_err();
            let excessive = 1.0 < pct_err;
            if excessive || print_anyway {
                let abs_diff = self.abs_diff();
                if excessive {
                    eprintln!("Excessive error:");
                } else {
                    eprintln!("Error summary:");
                }
                eprintln!("    expected: {:.15}", self.expected);
                eprintln!("    actual  : {:.15}", self.actual);
                eprintln!("    abs_diff: {abs_diff}");
                eprintln!("    pct_err:  {pct_err:2.1} %");
            }
            !excessive
        }
    }

    #[test]
    fn t2_u128_to_f64() {
        #[allow(dead_code)]
        const F: f64 = u128_to_f64(0);
        const_assert_eq!(F, 0.0_f64);

        let mut ae = AE {
            actual: 0.0_f64,
            expected: 0.0_f64,
        };

        let zero_f64 = u128_to_f64(0);
        assert_debug_snapshot!(zero_f64.floor(), @"0.0");
        assert_debug_snapshot!(zero_f64.ceil(), @"0.0");
        ae.actual = zero_f64;
        ae.expected = 0.0_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let u53_max = (1_u128 << 53) - 1;
        let u53_max_f64 = u128_to_f64(u53_max);
        ae.actual = u53_max_f64;
        ae.expected = 9007199254740991_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let two53 = 1_u128 << 53;
        let two53_f64 = u128_to_f64(two53);
        ae.actual = two53_f64;
        ae.expected = 9007199254740992_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let two127_f64 = u128_to_f64(1_u128 << 127);
        ae.actual = two127_f64;
        ae.expected = 170141183460469231731687303715884105728_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let u128_max_f64 = u128_to_f64(u128::MAX);
        ae.actual = u128_max_f64;
        ae.expected = 340282366920938463463374607431768211455_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let u128maxp1_over_two127 = (u128_max_f64 + 1.0) / two127_f64;
        ae.actual = u128maxp1_over_two127;
        ae.expected = 2.0;
        assert!(ae.pct_err_is_acceptable(false));

        ae.actual = two127_f64 / two53_f64;
        ae.expected = u128_to_f64(1_u128 << (127 - 53));
        assert!(ae.pct_err_is_acceptable(false));
    }

    #[test]
    fn t3_i128_to_f64() {
        #[allow(dead_code)]
        const F: f64 = i128_to_f64(0);
        const_assert_eq!(F, 0.0_f64);

        let mut ae = AE {
            actual: 0.0_f64,
            expected: 0.0_f64,
        };

        let zero_f64 = i128_to_f64(0);
        assert_debug_snapshot!(zero_f64.floor(), @"0.0");
        assert_debug_snapshot!(zero_f64.ceil(), @"0.0");
        ae.actual = zero_f64;
        ae.expected = 0.0_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let neg1_f64 = i128_to_f64(-1);
        assert_debug_snapshot!(neg1_f64.floor(), @"-1.0");
        assert_debug_snapshot!(neg1_f64.ceil(), @"-1.0");
        ae.actual = neg1_f64;
        ae.expected = -1_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let u53_max = (1_i128 << 53) - 1;
        let u53_max_f64 = i128_to_f64(u53_max);
        assert_debug_snapshot!(u53_max_f64.floor() as u64, @"9007199254740991");
        assert_debug_snapshot!(u53_max_f64.ceil() as u64, @"9007199254740991");
        ae.actual = u53_max_f64;
        ae.expected = 9007199254740991.0_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let two53 = 1_i128 << 53;
        let two53_f64 = i128_to_f64(two53);
        assert_debug_snapshot!(two53_f64.floor() as u64, @"9007199254740992");
        assert_debug_snapshot!(two53_f64.ceil() as u64, @"9007199254740992");
        ae.actual = two53_f64;
        ae.expected = 9007199254740992_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let i128_max_f64 = i128_to_f64(i128::MAX);
        ae.actual = i128_max_f64;
        ae.expected =
            65536.0_f64 * 65536.0 * 65536.0 * 65536.0 * 65536.0 * 65536.0 * 65536.0 * 32768.0 - 1.0;
        assert!(ae.pct_err_is_acceptable(false));

        let i128_min_f64 = i128_to_f64(i128::MIN);
        ae.actual = i128_min_f64;
        ae.expected = -170141183460469231731687303715884105728_f64;
        assert!(ae.pct_err_is_acceptable(false));

        let two126_f64 = i128_to_f64(1_i128 << 126);
        ae.actual = two126_f64;
        ae.expected =
            65536.0_f64 * 65536.0 * 65536.0 * 65536.0 * 65536.0 * 65536.0 * 65536.0 * 16384.0;
        assert!(ae.pct_err_is_acceptable(false));

        let i128maxp1_over_two126 = (i128_max_f64 + 1.0) / two126_f64;
        ae.actual = i128maxp1_over_two126;
        ae.expected = 2.0;
        assert!(ae.pct_err_is_acceptable(false));

        ae.actual = two126_f64 / two53_f64;
        ae.expected = i128_to_f64(1_i128 << (126 - 53));
        assert!(ae.pct_err_is_acceptable(false));
    }
}
