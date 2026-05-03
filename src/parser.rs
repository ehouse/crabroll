use crate::types::{Expr, Op, Token};
use chumsky::prelude::*;

/// Combines a left-hand expression and an (op, right) pair into a Binary node.
/// Passed by name to `foldl` so it can be shared between the factor and expr parsers.
fn fold_binary(left: Expr, (op, right): (Op, Expr)) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Builds a parser for dice expressions.
/// Handles standard arithmetic operators with correct precedence, dice rolls, and parenthesised groups.
/// Returns an `Expr` tree ready to be passed to the evaluator.
pub fn parser<'src>() -> impl Parser<'src, &'src [Token], Expr, extra::Err<Simple<'src, Token>>> {
    recursive(|expr| {
        // Matches a single number token and lifts it into an Expr::Number.
        let number = select! { Token::Number(n) => Expr::Number(n) };

        // Matches a single die token and lifts it into an Expr::Roll.
        let die = select! { Token::Die(kind, n, sides) => Expr::Roll { kind, n, sides } };

        // Matches a parenthesised expression, recursing back to the top of the grammar.
        // This is what gives parens their precedence-override behaviour.
        let group = expr.delimited_by(just(Token::LParen), just(Token::RParen));

        // TERM: the smallest unit of an expression: a die roll, a bare number, or a grouped sub-expression.
        let term = die.or(number).or(group);

        // FACTOR: handles * and / with left-associativity.
        // Parses a term, then folds zero or more (* term) or (/ term) pairs into it,
        // producing a left-leaning Binary tree. Higher precedence than + and -.
        // Example: `2 * 3 * 4` becomes Binary(*, Binary(*, 2, 3), 4).
        let factor = term.clone().foldl(
            just(Token::Star)
                .to(Op::Mul)
                .or(just(Token::Slash).to(Op::Div))
                .then(term)
                .repeated(),
            fold_binary,
        );

        // EXP: handles + and - with left-associativity.
        // Parses a factor, then folds zero or more (+ factor) or (- factor) pairs into it.
        // Lower precedence than * and /, so `2 + 3 * 4` correctly parses as `2 + (3 * 4)`
        // because the `3 * 4` is fully resolved as a factor before expr sees it.
        factor.clone().foldl(
            just(Token::Plus)
                .to(Op::Add)
                .or(just(Token::Minus).to(Op::Sub))
                .then(factor)
                .repeated(),
            fold_binary,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RollKind;

    fn parse(tokens: &[Token]) -> Option<Expr> {
        parser().parse(tokens).into_output()
    }

    #[test]
    fn test_addition() {
        let tokens = [Token::Number(1.0), Token::Plus, Token::Number(2.0)];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(expr, Expr::Binary { op: Op::Add, .. }));
    }

    #[test]
    fn test_number() {
        let tokens = [Token::Number(42.0)];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(expr, Expr::Number(n) if n == 42.0));
    }

    #[test]
    fn test_subtraction() {
        let tokens = [Token::Number(1.0), Token::Minus, Token::Number(2.0)];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(expr, Expr::Binary { op: Op::Sub, .. }));
    }

    #[test]
    fn test_multiplication() {
        let tokens = [Token::Number(2.0), Token::Star, Token::Number(3.0)];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(expr, Expr::Binary { op: Op::Mul, .. }));
    }

    #[test]
    fn test_division() {
        let tokens = [Token::Number(6.0), Token::Slash, Token::Number(2.0)];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(expr, Expr::Binary { op: Op::Div, .. }));
    }

    #[test]
    fn test_die() {
        let tokens = [Token::Die(RollKind::Normal, 2, 6)];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(
            expr,
            Expr::Roll {
                kind: RollKind::Normal,
                n: 2,
                sides: 6
            }
        ));
    }

    #[test]
    fn test_advantage() {
        let tokens = [Token::Die(RollKind::Advantage, 1, 20)];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(
            expr,
            Expr::Roll {
                kind: RollKind::Advantage,
                n: 1,
                sides: 20
            }
        ));
    }

    #[test]
    fn test_disadvantage() {
        let tokens = [Token::Die(RollKind::Disadvantage, 1, 20)];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(
            expr,
            Expr::Roll {
                kind: RollKind::Disadvantage,
                n: 1,
                sides: 20
            }
        ));
    }

    #[test]
    fn test_grouped_expression() {
        let tokens = [
            Token::LParen,
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::RParen,
            Token::Star,
            Token::Number(3.0),
        ];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(expr, Expr::Binary { op: Op::Mul, .. }));
    }

    #[test]
    fn test_die_in_expression() {
        let tokens = [
            Token::Die(RollKind::Normal, 2, 6),
            Token::Plus,
            Token::Number(3.0),
        ];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(expr, Expr::Binary { op: Op::Add, .. }));
    }

    #[test]
    fn test_chained_addition() {
        let tokens = [
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::Plus,
            Token::Number(3.0),
        ];
        let expr = parse(&tokens).expect("should parse");
        assert!(matches!(expr, Expr::Binary { op: Op::Add, .. }));
    }
}
