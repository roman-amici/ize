mod deck_io;

pub mod prelude {
    pub use super::deck_io::load_deck;
    pub use super::deck_io::save_deck;
    pub use super::deck_io::load_practice_run;
    pub use super::Card;
}

pub struct PracticeRun {
    pub deck_path : String,
    pub remaining : Vec<usize>,
    pub memorized : Vec<usize>,
    pub needs_work : Vec<usize>,
    pub incorrect : Vec<usize>,
}

impl PracticeRun {
    pub fn new() -> Self {
        PracticeRun {
            deck_path : "".to_string(),
            remaining : vec![],
            memorized : vec![],
            needs_work : vec![],
            incorrect : vec![]
        }
    }
}

pub struct Deck {
    cards : Vec<Card>
}

pub struct Card {
    pub card_id : usize,
    pub front : String,
    pub back : String
}