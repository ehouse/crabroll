use crate::types::{RollKind, Token};
use chumsky::prelude::*;

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Token>, extra::Err<Simple<'src, char>>> {
    // Parses a base-10 integer as Token::Number(f64) for standalone numbers in expressions.
    let number = text::int::<_, extra::Err<Simple<'src, char>>>(10)
        .map(|s: &str| Token::Number(s.parse().unwrap()));

    // Parses a base-10 integer as a raw i32. Used internally by the die parser for n and sides.
    let integer =
        text::int::<_, extra::Err<Simple<'src, char>>>(10).map(|s: &str| s.parse::<i32>().unwrap());

    // Each operator matches a single character and replaces it with a constant Token variant.
    let plus = just('+').to(Token::Plus);
    let minus = just('-').to(Token::Minus);
    let star = just('*').to(Token::Star);
    let slash = just('/').to(Token::Slash);

    // Matches an optional die prefix: a count (e.g. `2` in `2d6`), `^` for advantage, or `v` for
    // disadvantage. Produces (RollKind, n). Absent prefix defaults to (Normal, 1) in the die parser.
    let prefix = choice((
        integer.map(|n| (RollKind::Normal, n)),
        just('^').to((RollKind::Advantage, 1)),
        just('v').to((RollKind::Disadvantage, 1)),
    ))
    .or_not();

    // Matches a full die expression: optional prefix, `d`, then the number of sides.
    // Examples: `2d6`, `d20`, `^d20`, `vd20`.
    let die = prefix
        .then(just('d'))
        .then(integer)
        .map(|((prefix, _), sides)| {
            let (kind, n) = prefix.unwrap_or((RollKind::Normal, 1));
            Token::Die(kind, n, sides as u32)
        });

    choice((die, number, plus, minus, star, slash))
        .padded()
        .repeated()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        lexer().parse(input).unwrap()
    }

    #[test]
    fn test_number() {
        assert!(matches!(lex("42")[0], Token::Number(n) if n == 42.0));
    }

    #[test]
    fn test_plus() {
        assert!(matches!(lex("+")[0], Token::Plus));
    }

    #[test]
    fn test_minus() {
        assert!(matches!(lex("-")[0], Token::Minus));
    }

    #[test]
    fn test_star() {
        assert!(matches!(lex("*")[0], Token::Star));
    }

    #[test]
    fn test_slash() {
        assert!(matches!(lex("/")[0], Token::Slash));
    }

    #[test]
    fn test_number_plus_number() {
        let tokens = lex("1 + 2");
        assert!(matches!(tokens[0], Token::Number(n) if n == 1.0));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Number(n) if n == 2.0));
    }

    #[test]
    fn test_die_simple() {
        assert!(matches!(lex("d20")[0], Token::Die(RollKind::Normal, 1, 20)));
    }

    #[test]
    fn test_die_with_count() {
        assert!(matches!(lex("2d6")[0], Token::Die(RollKind::Normal, 2, 6)));
    }

    #[test]
    fn test_die_advantage() {
        assert!(matches!(
            lex("^d20")[0],
            Token::Die(RollKind::Advantage, 1, 20)
        ));
    }

    #[test]
    fn test_die_disadvantage() {
        assert!(matches!(
            lex("vd20")[0],
            Token::Die(RollKind::Disadvantage, 1, 20)
        ));
    }

    #[test]
    fn test_full_expression() {
        let tokens = lex("2d6 + 3");
        assert!(matches!(tokens[0], Token::Die(RollKind::Normal, 2, 6)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Number(n) if n == 3.0));
    }
}
