use miette::Result;
use std::time::Instant;
use clap::Parser;

use tort::args::Args;
use tort::syntax::{self, Line};
use tort::quiz::QuizMachine;

fn main() -> Result<()> {
    let start_time = Instant::now();
    let args = Args::parse();

    let mut lines = Vec::<Line>::new();
    for path in args.files {
        let source_name = path.display().to_string();
        let source = std::fs::read_to_string(path).expect("can't read the input file");
        let mut parser = syntax::Parser::new(&source_name, &source);
        lines.append(&mut parser.parse()?);
    }

    if !args.check {
        let machine = QuizMachine::new(args.random, args.number_of_tests.unwrap_or_default(), start_time);
        machine.append(&mut lines);
        machine.run().expect("can't run the quiz");
    }

    Ok(())
}
