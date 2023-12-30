use std::{env, error::Error};

use ize_core::{prelude::*, Deck};
use rustyline::{history::FileHistory, Editor};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut rl = rustyline::DefaultEditor::new()?;

    if args.len() > 1 {
        let run_path = &args[1];

        let (run, deck) = load_practice_run(run_path)?;
        practice_run(&mut rl, run, deck)?;
    }

    loop {
        println!("Load deck? yes, no");
        let line = rl.readline(">>")?;

        if line == "no" || line == "n" || line == "quit" || line == "q" {
            return Ok(())
        }

        println!("Enter path:");
        let line = rl.readline(">>")?;

        let deck = load_deck(&line)?;
        let run = PracticeRun::new_from_deck(&deck);

        practice_run(&mut rl, run, deck)?;

    }

}

fn setup_run(category : RunCategory, run : &mut PracticeRun) {
    run.move_category(category, RunCategory::Remaining);

    run.shuffle(RunCategory::Remaining);
}

fn setup_run_all(run : &mut PracticeRun) {
    run.move_category(RunCategory::Incorrect, RunCategory::Remaining);
    run.move_category(RunCategory::Working, RunCategory::Remaining);
    run.move_category(RunCategory::Memorized, RunCategory::Remaining);

    run.shuffle(RunCategory::Remaining);
}

fn category_choice(rl : &mut Editor<(), FileHistory>, run : &mut PracticeRun) -> Result<bool, Box<dyn Error>> 
{
    loop {
        let incorrect_len = run.incorrect.len();
        let working_len = run.working.len();
        let memorized_len = run.memorized.len();

        let all = run.remaining.len() + incorrect_len + working_len + memorized_len;
    
        println!("1: all ({}), 2: incorrect ({}), 3: working ({}), 4: memorized ({}), or q to quit", all, incorrect_len, working_len, memorized_len);
    
        let line = rl.readline(">>")?;

        if line == "q" || line == "quit" {
            return Ok(false);
        }

        if let Ok(num) = line.parse::<usize>() {
            match num {
                1 => setup_run_all(run),
                2 => setup_run(RunCategory::Incorrect, run),
                3 => setup_run(RunCategory::Remaining, run),
                4 => setup_run(RunCategory::Memorized, run),
                _ =>  {
                    println!("Unknown command '{}'", line);
                    continue;
                },
            }

            return Ok(true);
        } else {
            println!("Unknown command '{}'", line);
        }

    }

}

fn next_card(rl : &mut Editor<(), FileHistory>, run : &mut PracticeRun, deck : &Deck) -> Result<bool, Box<dyn Error>> {
    let card_id = run.remaining.last().expect("Next card called on empty deck.");

    let card = deck.cards.get(card_id).expect("Card not found in deck.");

    println!("{}", card.front);

    let line = rl.readline("< space >")?;

    if line == "q" || line == "quit" {
        return Ok(false);
    }

    println!("{}", card.back);

    card_choice(rl, *card_id, run)
}

fn card_choice(rl : &mut Editor<(), FileHistory>, card_id : usize, run : &mut PracticeRun) -> Result<bool, Box<dyn Error>> {

    loop {
        println!("How'd you do?");
        println!("1: skip, 2: incorrect, 3: working, 4: memorized, or quit?");

        let line = rl.readline(">>")?;

        if line == "q" || line == "quit" {
            return Ok(false);
        }

        if let Ok(choice) = line.parse::<usize>() {
            match choice {
                1 => run.skip(),
                2 => run.move_index(card_id, RunCategory::Remaining, RunCategory::Incorrect)?,
                3 => run.move_index(card_id, RunCategory::Remaining, RunCategory::Working)?,
                4 => run.move_index(card_id, RunCategory::Remaining, RunCategory::Memorized)?,
                _ => {
                    println!("Unknown command {}", line);
                    continue;
                } 
            }
        }

        return Ok(true);
    }
}

fn save_run_prompt(rl: &mut Editor<(), FileHistory>, run : &mut PracticeRun) -> Result<(), Box<dyn Error>> {
    println!("Would you like to save paractice run? yes, no");
    let line = rl.readline(">>")?;
    if line == "no" || line == "n" {
        return Ok(())
    }

    let path = rl.readline_with_initial(">>", (&run.last_save,""))?;

    if path.is_empty() {
        return Ok(());
    }

    save_practice_run(&path, run)
}

fn practice_run(rl : &mut Editor<(), FileHistory>, mut run: PracticeRun, deck: Deck) -> Result<(), Box<dyn Error>> {
    loop {

        if run.remaining.is_empty() {
            println!("No cards remaining would you like to start over?");

            if category_choice(rl, &mut run)? {
                
            } else {
                break;
            }
        }

        if !next_card( rl, &mut run, &deck)? {
            break
        }
    }

    save_run_prompt(rl, &mut run)?;
    Ok(())
}
