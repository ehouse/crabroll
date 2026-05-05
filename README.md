# crabroll

Dice expression evaluator written in Rust. Parses and evaluates standard dice notation with arithmetic, advantage, and disadvantage. Planned WASM build for embedding in websites.

## Usage

Run with `cargo run`. The REPL accepts standard dice notation: `2d6`, `d20`, `^d20` (advantage), `vd20` (disadvantage), combined with arithmetic and grouping. Each result shows the rolled value plus theoretical min, max, and average.

```
> 2d6 + 3
11 (min: 5, max: 15, avg: 10.00)
> ^d20
18 (min: 1, max: 20, avg: 13.83)
> quit
```

## Tests

```
cargo test
```
