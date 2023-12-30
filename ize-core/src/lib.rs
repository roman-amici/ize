mod deck_io;
mod run_actions;

pub mod prelude {
    pub use super::deck_io::load_deck;
    pub use super::deck_io::load_practice_run;
    pub use super::deck_io::save_deck;
    pub use super::run_actions::*;
    pub use super::Card;
    pub use super::PracticeRun;
}

pub struct PracticeRun {
    pub deck_path: String,
    pub remaining: Vec<usize>,
    pub memorized: Vec<usize>,
    pub working: Vec<usize>,
    pub incorrect: Vec<usize>,
}

impl PracticeRun {
    pub fn new() -> Self {
        PracticeRun {
            deck_path: "".to_string(),
            remaining: vec![],
            memorized: vec![],
            working: vec![],
            incorrect: vec![],
        }
    }
}

pub struct Deck {
    cards: Vec<Card>,
}

pub struct Card {
    pub card_id: usize,
    pub front: String,
    pub back: String,
}
