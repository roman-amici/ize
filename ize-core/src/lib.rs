use std::collections::HashMap;

mod deck_io;
mod run_actions;

pub mod prelude {
    pub use super::deck_io::load_deck;
    pub use super::deck_io::load_practice_run;
    pub use super::deck_io::save_deck;
    pub use super::deck_io::save_practice_run;
    pub use super::run_actions::*;
    pub use super::Card;
    pub use super::PracticeRun;
}

pub struct PracticeRun {
    pub deck_path: String,
    pub last_save: String,

    pub remaining: Vec<usize>,
    pub memorized: Vec<usize>,
    pub working: Vec<usize>,
    pub incorrect: Vec<usize>,
}

impl PracticeRun {
    pub fn new() -> Self {
        PracticeRun {
            deck_path: "".to_string(),
            last_save: "".to_string(),
            remaining: vec![],
            memorized: vec![],
            working: vec![],
            incorrect: vec![],
        }
    }

    pub fn new_from_deck(deck: &Deck) -> Self {
        let mut run = Self::new();

        run.remaining = deck.cards.iter().map(|(id, _)| *id).collect();
        run.shuffle(run_actions::RunCategory::Remaining);

        run
    }
}

pub struct Deck {
    pub cards: HashMap<usize, Card>,
}

pub struct Card {
    pub card_id: usize,
    pub front: String,
    pub back: String,
}
