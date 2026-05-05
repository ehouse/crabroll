use crate::types::{EvalResult, Expr, Op, RollKind};

use rand::RngExt;

// Recursively evaluates an Expr tree, rolling any dice and computing the total value and stats.
// Binary nodes evaluate both sides first, then combine based on the operator.
// Stats (min/max/avg) propagate through the tree
pub fn evaluate(expr: Expr) -> EvalResult {
    match expr {
        // Value and all stats are just the number itself.
        Expr::Number(n) => EvalResult {
            expr: Expr::Number(n),
            value: n,
            min: n,
            max: n,
            avg: n,
        },

        // Roll n dice of `sides` sides, sum them.
        // Advantage/disadvantage always rolls 2 dice and takes the highest/lowest.
        // Stats: min = n, max = n * sides, avg = n * (1 + sides) / 2.
        Expr::Roll {
            kind,
            n,
            sides,
            results: _,
        } => {
            let mut rng = rand::rng();
            // Generate either normal or advantage/disadvantage rolls in  a set
            let results: Vec<i32> = match kind {
                // Generate and map over random range of values
                RollKind::Normal => (0..n.abs())
                    .map(|_| rng.random_range(1..=sides) as i32)
                    .collect(),
                // Simple since we're always rolling 2 dice
                RollKind::Advantage | RollKind::Disadvantage => {
                    (0..2).map(|_| rng.random_range(1..=sides) as i32).collect()
                }
            };

            let value = match kind {
                RollKind::Normal => results.iter().sum::<i32>() as f64,
                RollKind::Advantage => results[0].max(results[1]) as f64,
                RollKind::Disadvantage => results[0].min(results[1]) as f64,
            };

            // Generating averages for rolls is a nightmare, especially advantage/disadvantage
            let avg = match kind {
                RollKind::Normal => n as f64 * (1.0 + sides as f64) / 2.0,
                RollKind::Advantage => {
                    let s = sides as f64;
                    // For each possible outcome k (1 to sides), compute the probability that
                    // the highest of two dice equals exactly k using a Cumulative Distribution Function difference:
                    // P(max = k) = P(both <= k) - P(both <= k-1) = (k/s)^2 - ((k-1)/s)^2
                    // Multiply each outcome by its probability and sum to get the expected value.
                    (1..=sides as i32).fold(0.0, |sum, k| {
                        sum + k as f64 * ((k as f64 / s).powi(2) - ((k - 1) as f64 / s).powi(2))
                    })
                }
                RollKind::Disadvantage => {
                    let s = sides as f64;
                    // Same CDF approach but for the minimum of two dice.
                    // P(min = k) = P(both >= k) - P(both >= k+1) = ((s-k+1)/s)^2 - ((s-k)/s)^2
                    (1..=sides as i32).fold(0.0, |sum, k| {
                        let p = ((s - (k - 1) as f64) / s).powi(2) - ((s - k as f64) / s).powi(2);
                        sum + k as f64 * p
                    })
                }
            };

            EvalResult {
                expr: Expr::Roll {
                    kind,
                    n,
                    sides,
                    results: Some(results),
                },
                value,
                min: n as f64,
                max: n as f64 * sides as f64,
                avg,
            }
        }

        // Evaluate both sides first, then combine.
        // The evaluated left/right are stored back into the tree so the display
        // code can show individual roll results (e.g. `{ 3 5 } + 2`).
        Expr::Binary { op, left, right } => {
            let left = evaluate(*left);
            let right = evaluate(*right);

            let value = match op {
                Op::Add => left.value + right.value,
                Op::Sub => left.value - right.value,
                Op::Mul => left.value * right.value,
                Op::Div => left.value / right.value,
            };

            let (min, max, avg) = match op {
                Op::Add => (
                    left.min + right.min,
                    left.max + right.max,
                    left.avg + right.avg,
                ),
                Op::Sub => (
                    left.min - right.max,
                    left.max - right.min,
                    left.avg - right.avg,
                ),
                Op::Mul => (
                    left.min * right.min,
                    left.max * right.max,
                    left.avg * right.avg,
                ),
                Op::Div => (
                    left.min / right.max,
                    left.max / right.min,
                    left.avg / right.avg,
                ),
            };

            EvalResult {
                expr: Expr::Binary {
                    op,
                    left: Box::new(left.expr),
                    right: Box::new(right.expr),
                },
                value,
                min,
                max,
                avg,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(n: f64) -> Expr {
        Expr::Number(n)
    }

    fn binary(op: Op, l: f64, r: f64) -> EvalResult {
        evaluate(Expr::Binary {
            op,
            left: Box::new(num(l)),
            right: Box::new(num(r)),
        })
    }

    #[test]
    fn test_number() {
        let r = evaluate(num(7.0));
        assert_eq!(r.value, 7.0);
        assert_eq!(r.min, 7.0);
        assert_eq!(r.max, 7.0);
        assert_eq!(r.avg, 7.0);
    }

    #[test]
    fn test_add() {
        let r = binary(Op::Add, 3.0, 4.0);
        assert_eq!(r.value, 7.0);
        assert_eq!(r.min, 7.0);
        assert_eq!(r.max, 7.0);
    }

    #[test]
    fn test_sub() {
        let r = binary(Op::Sub, 10.0, 3.0);
        assert_eq!(r.value, 7.0);
    }

    #[test]
    fn test_mul() {
        let r = binary(Op::Mul, 3.0, 4.0);
        assert_eq!(r.value, 12.0);
    }

    #[test]
    fn test_div() {
        let r = binary(Op::Div, 12.0, 4.0);
        assert_eq!(r.value, 3.0);
    }

    #[test]
    fn test_roll_normal_stats() {
        let r = evaluate(Expr::Roll {
            kind: RollKind::Normal,
            n: 2,
            sides: 6,
            results: None,
        });
        assert_eq!(r.min, 2.0);
        assert_eq!(r.max, 12.0);
        assert_eq!(r.avg, 7.0);
        assert!(r.value >= 2.0 && r.value <= 12.0);
    }

    #[test]
    fn test_roll_advantage_stats() {
        let r = evaluate(Expr::Roll {
            kind: RollKind::Advantage,
            n: 1,
            sides: 20,
            results: None,
        });
        assert_eq!(r.min, 1.0);
        assert_eq!(r.max, 20.0);
        assert!((r.avg - 13.825).abs() < 0.001);
        assert!(r.value >= 1.0 && r.value <= 20.0);
    }

    #[test]
    fn test_roll_disadvantage_stats() {
        let r = evaluate(Expr::Roll {
            kind: RollKind::Disadvantage,
            n: 1,
            sides: 20,
            results: None,
        });
        assert_eq!(r.min, 1.0);
        assert_eq!(r.max, 20.0);
        assert!((r.avg - 7.175).abs() < 0.001);
        assert!(r.value >= 1.0 && r.value <= 20.0);
    }
}
