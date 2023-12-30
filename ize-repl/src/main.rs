use std::{env, error::Error};

use ize_core::{prelude::*, Deck};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let rl = rustyline::DefaultEditor::new()?;

    if args.len() > 1 {
        let run_path = &args[1];

        let (run, deck) = load_practice_run(run_path)?;
    }

    Ok(())
}

fn start_run(run: PracticeRun, deck: Deck) -> Result<(), Box<dyn Error>> {
    loop {}

    Ok(())
}
