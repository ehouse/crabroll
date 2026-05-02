use chumsky::Parser;
use crabroll::evaluator::evaluate;
use crabroll::lexer::lexer;
use crabroll::parser::parser;
use rustyline::DefaultEditor;

fn main() {
    println!("crabroll - type a dice expression or 'quit' to exit");

    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let input = match rl.readline("> ") {
            Ok(line) => line,
            Err(_) => break,
        };

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "quit" {
            break;
        }

        rl.add_history_entry(input).ok();

        let tokens = match lexer().parse(input).into_result() {
            Ok(t) => t,
            Err(e) => {
                println!("lex error: {:?}", e);
                continue;
            }
        };

        let ast = match parser().parse(tokens.as_slice()).into_result() {
            Ok(a) => a,
            Err(e) => {
                println!("parse error: {:?}", e);
                continue;
            }
        };

        let result = evaluate(ast);

        println!(
            "{} (min: {}, max: {}, avg: {:.2})",
            result.value, result.min, result.max, result.avg
        );
    }
}
