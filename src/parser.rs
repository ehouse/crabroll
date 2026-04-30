use crate::types::{Expr, Op, Token};
use chumsky::prelude::*;

pub fn parser<'src>() -> impl Parser<'src, &'src [Token], Expr, extra::Err<Simple<'src, Token>>> {
    // Matches a single number token and lifts it into an Expr::Number.
    let number = select! { Token::Number(n) => Expr::Number(n) };

    // TERM: the smallest unit of an expression -- a single atom with no operators.
    // Currently just numbers, but will grow to include dice rolls and parenthesised groups.
    let term = number;

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
        |left, (op, right)| Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
    );

    // EXP: handles + and - with left-associativity.
    // Parses a factor, then folds zero or more (+ factor) or (- factor) pairs into it.
    // Lower precedence than * and /, so `2 + 3 * 4` correctly parses as `2 + (3 * 4)`
    // because the `3 * 4` is fully resolved as a factor before expr sees it.
    let expr = factor.clone().foldl(
        just(Token::Plus)
            .to(Op::Add)
            .or(just(Token::Minus).to(Op::Sub))
            .then(factor)
            .repeated(),
        |left, (op, right)| Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
    );

    expr
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
    #[should_panic]
    fn test_die() {
        let tokens = [Token::Die(RollKind::Normal, 2, 6)];
        parse(&tokens).expect("die not yet implemented");
    }

    #[test]
    #[should_panic]
    fn test_advantage() {
        let tokens = [Token::Die(RollKind::Advantage, 1, 20)];
        parse(&tokens).expect("advantage not yet implemented");
    }

    #[test]
    #[should_panic]
    fn test_disadvantage() {
        let tokens = [Token::Die(RollKind::Disadvantage, 1, 20)];
        parse(&tokens).expect("disadvantage not yet implemented");
    }

    #[test]
    #[should_panic]
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
        parse(&tokens).expect("grouping not yet implemented");
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
