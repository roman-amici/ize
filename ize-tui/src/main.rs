use std::{env, error::Error};

mod file_explorer;
mod practice_run;
mod utils;

use cursive::{
    view::Nameable,
    views::{Button, Dialog, DummyView, LinearLayout},
    Cursive,
};
use file_explorer::show_file_explorer;
use ize_core::{prelude::*, Deck, PracticeRun};
use practice_run::begin_run;
use utils::show_error;

const MAIN_MENU: &str = "MainMenu";
const RUN_SELECT: &str = "RunSelect";

struct RunState {
    run_data: RunData,
    card_content_state: CardContentState,
    count: usize,
}

struct RunData {
    run: PracticeRun,
    deck: Deck,
}

impl RunData {
    pub fn current_card_front(&self) -> Option<String> {
        let current_card_id = self.run.remaining.last()?;
        Some(self.deck.cards[current_card_id].front.clone())
    }

    pub fn current_card_back(&self) -> Option<String> {
        let current_card_id = self.run.remaining.last().unwrap();
        Some(self.deck.cards[current_card_id].back.clone())
    }

    pub fn is_done(&self) -> bool {
        self.run.remaining.last().is_none()
    }
}

#[derive(Copy, Clone, Debug)]
enum CardContentState {
    Front,
    Back,
}

fn try_load_args(siv : &mut Cursive, path : &str) -> Result<(), Box<dyn Error>> {
    // First try loading it as a run.
   let result = load_run_state(path);

   // If that doesn't work load it as a deck.
   let run_state = if let Ok(run_state) = result {
    run_state
   } else {
    new_run_state(path).unwrap()
   };

   siv.pop_layer();
   siv.set_user_data(run_state);
   begin_run(siv);

   Ok(())
}

fn main() {
    let mut siv = cursive::default();

    let args : Vec<String> = env::args().collect();
    if args.len() >= 2 && try_load_args(&mut siv, &args[1]).is_ok() {
    }
    else {
        main_menu(&mut siv);
    }

    siv.run();
}

fn main_menu(siv: &mut Cursive) {
    siv.pop_layer();

    let buttons = LinearLayout::vertical()
        .child(Button::new("New Deck", new_deck))
        .child(Button::new("Edit Deck", edit_deck))
        .child(Button::new("New Run", new_run))
        .child(Button::new("Resume Run", resume_run))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    let layer = Dialog::around(buttons)
        .title("Select Activity")
        .with_name(MAIN_MENU);

    siv.add_layer(layer);
}

fn new_deck(siv: &mut Cursive) {}

fn edit_deck(siv: &mut Cursive) {}

fn file_explorer_load_deck_new_run(siv: &mut Cursive, file_name: &str) {
    let run_state = new_run_state(file_name);

    match run_state {
        Err(e) => {
            show_error(siv, e.as_ref());
        }
        Ok(run_state) => {
            siv.pop_layer();
            siv.set_user_data(run_state);
            begin_run(siv);
        }
    }
}

fn file_explorer_resume_run(siv: &mut Cursive, file_name: &str) {
    let run_state = load_run_state(file_name);

    match run_state {
        Err(e) => {
            show_error(siv, e.as_ref());
        }
        Ok(run_state) => {
            siv.pop_layer();
            siv.set_user_data(run_state);
            begin_run(siv);
        }
    }
}

fn new_run(siv: &mut Cursive) {
    show_file_explorer(
        siv,
        "./".to_string(),
        Box::new(file_explorer_load_deck_new_run),
        Box::new(|s| {
            s.pop_layer();
        }),
    );
}

fn resume_run(siv: &mut Cursive) {
    show_file_explorer(
        siv,
        "./".to_string(),
        Box::new(file_explorer_resume_run),
        Box::new(|s| {
            s.pop_layer();
        }),
    );
}

fn load_run_state(run_path: &str) -> Result<RunState, Box<dyn Error>> {
    let (run, deck) = load_practice_run(run_path)?;

    let run_data = RunData { deck, run };
    Ok(RunState {
        card_content_state: CardContentState::Front,
        run_data,
        count: 0,
    })
}

fn new_run_state(deck_path: &str) -> Result<RunState, Box<dyn Error>> {
    let deck = load_deck(deck_path)?;

    let mut run = PracticeRun::new_from_deck(&deck);
    run.deck_path = deck_path.to_string();
    let run_data = RunData { deck, run };

    Ok(RunState {
        card_content_state: CardContentState::Front,
        run_data,
        count: 0,
    })
}
