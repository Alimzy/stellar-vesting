//! Pure vesting arithmetic, kept free of `Env` so it is trivially testable.

use crate::error::Error;

/// Amount vested at `now` for a schedule of `total` tokens.
///
/// * before `cliff`: 0
/// * from `end` on: `total`
/// * in between: `total * (now - start) / (end - start)`, rounded down
///
/// Callers must have validated `start <= cliff <= end` and `start < end`.
/// The result is always in `0..=total` and non-decreasing in `now`.
pub fn vested(total: i128, start: u64, cliff: u64, end: u64, now: u64) -> Result<i128, Error> {
    if now < cliff {
        return Ok(0);
    }
    if now >= end {
        return Ok(total);
    }
    // now >= cliff >= start, and now < end, so both values are positive.
    let elapsed = i128::from(now - start);
    let duration = i128::from(end - start);
    total
        .checked_mul(elapsed)
        .ok_or(Error::MathOverflow)?
        .checked_div(duration)
        .ok_or(Error::MathOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_before_cliff() {
        assert_eq!(vested(1_000, 0, 100, 1_000, 99), Ok(0));
    }

    #[test]
    fn jumps_to_linear_amount_at_cliff() {
        // 10% of the period has elapsed at the cliff.
        assert_eq!(vested(1_000, 0, 100, 1_000, 100), Ok(100));
    }

    #[test]
    fn linear_in_the_middle_and_rounds_down() {
        assert_eq!(vested(1_000, 0, 0, 1_000, 500), Ok(500));
        assert_eq!(vested(10, 0, 0, 3, 1), Ok(3));
    }

    #[test]
    fn full_amount_at_and_after_end() {
        assert_eq!(vested(1_000, 0, 100, 1_000, 1_000), Ok(1_000));
        assert_eq!(vested(1_000, 0, 100, 1_000, u64::MAX), Ok(1_000));
    }

    #[test]
    fn cliff_equal_to_end_is_all_or_nothing() {
        assert_eq!(vested(1_000, 0, 1_000, 1_000, 999), Ok(0));
        assert_eq!(vested(1_000, 0, 1_000, 1_000, 1_000), Ok(1_000));
    }

    #[test]
    fn overflow_is_reported_not_wrapped() {
        assert_eq!(
            vested(i128::MAX, 0, 0, u64::MAX, u64::MAX - 1),
            Err(Error::MathOverflow)
        );
    }
}
