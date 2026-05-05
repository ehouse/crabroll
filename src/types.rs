#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, PartialEq, Debug)]
pub enum RollKind {
    Advantage,
    Disadvantage,
    Normal,
}

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Roll {
        kind: RollKind,
        n: i32,
        sides: u32,
        results: Option<Vec<i32>>,
    },
    Binary {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

pub struct EvalResult {
    pub expr: Expr,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Number(f64),
    Die(RollKind, i32, u32), // kind, n, sides
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}
