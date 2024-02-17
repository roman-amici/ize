use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Lines, Write},
    iter::Peekable,
};

use crate::{Card, Deck, PracticeRun};

#[derive(Debug)]
struct ParsingError {
    error_message: String,
}

impl ParsingError {
    pub fn new(error_message: String) -> Self {
        ParsingError { error_message }
    }

    pub fn box_new(error_message: String) -> Box<Self> {
        Box::new(ParsingError::new(error_message))
    }
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.error_message)
    }
}

impl Error for ParsingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

fn scan_or_error(
    reader: &mut Peekable<Lines<BufReader<File>>>,
    error_msg: &str,
) -> Result<(), Box<dyn Error>> {
    if !scan(reader)? {
        Err(ParsingError::box_new(error_msg.to_string()))
    } else {
        Ok(())
    }
}

fn scan(reader: &mut Peekable<Lines<BufReader<File>>>) -> Result<bool, Box<dyn Error>> {
    // Iterate until we hit a non-empty line

    loop {
        if let Some(line) = reader.peek() {
            match line.as_ref() {
                Ok(s) => {
                    if !s.trim().is_empty() {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        } else {
            return Ok(false);
        }

        // Safe to unwrap because we would have returned already if the iteration expired
        reader.next().unwrap()?;
    }
}

fn read_id(reader: &mut Peekable<Lines<BufReader<File>>>) -> Result<usize, Box<dyn Error>> {
    if let Some(line) = reader.next() {
        let line = line?;

        if let Ok(id) = line.parse() {
            Ok(id)
        } else {
            Err(ParsingError::box_new(format!(
                "Card id {line} must be a number."
            )))
        }
    } else {
        Err(ParsingError::box_new(
            "Deck file invalid. Expected card id.".to_string(),
        ))
    }
}

fn read_text(reader: &mut Peekable<Lines<BufReader<File>>>) -> Result<String, Box<dyn Error>> {
    if let Some(line) = reader.next() {
        let line = line?;
        Ok(line)
    } else {
        Err(ParsingError::box_new("Expected card content".to_string()))
    }
}

fn read_card(
    reader: &mut Peekable<Lines<BufReader<File>>>,
) -> Result<Option<Card>, Box<dyn Error>> {
    // Done iterating
    if !scan(reader)? {
        return Ok(None);
    }

    let card_id = read_id(reader)?;
    let front = read_text(reader)?;
    let back = read_text(reader)?;

    Ok(Some(Card {
        front,
        back,
        card_id,
    }))
}

pub fn load_deck(deck_path: &str) -> Result<Deck, Box<dyn Error>> {
    let file = File::open(deck_path)?;

    let mut iter = BufReader::new(file).lines().into_iter().peekable();

    let mut cards = HashMap::<usize, Card>::new();
    while {
        if let Some(card) = read_card(&mut iter)? {
            // If there are duplicates, onlt the last one will be taken.
            cards.insert(card.card_id, card);
            true
        } else {
            false
        }
    } {}

    Ok(Deck { cards })
}

pub fn save_deck(filepath: &str, deck: Deck) -> Result<(), Box<dyn Error>> {
    //Todo: write atomically...
    let file = File::create(filepath)?;
    let mut writer = BufWriter::new(&file);

    writeln!(&mut writer, "")?;

    for (_, card) in deck.cards {
        writeln!(&mut writer, "{}", card.card_id)?;
        writeln!(&mut writer, "{}", &card.front)?;
        writeln!(&mut writer, "{}", &card.back)?;
        writeln!(&mut writer, "")?;
    }

    Ok(())
}

fn read_id_list(
    reader: &mut Peekable<Lines<BufReader<File>>>,
) -> Result<Vec<usize>, Box<dyn Error>> {
    let mut vec = vec![];
    loop {
        if let Some(line) = reader.peek() {
            match line.as_ref() {
                Ok(line) => {
                    if let Ok(parsed) = line.parse() {
                        vec.push(parsed);
                        // Safe to unwrap due to the prior peek succeeding.
                        _ = reader.next().unwrap();
                    } else {
                        // return on the first failed parse
                        return Ok(vec);
                    }
                }
                _ => {}
            }
        } else {
            // Return on iteration end
            return Ok(vec);
        }
    }
}

fn load_practice_run_file(filepath: &str) -> Result<PracticeRun, Box<dyn Error>> {
    let file = File::open(filepath)?;
    let mut iter = BufReader::new(file).lines().into_iter().peekable();
    let reader = &mut iter;

    let mut run = PracticeRun::new();
    run.last_save = fs::canonicalize(filepath)?.to_str().unwrap().to_string();

    scan_or_error(reader, "Expected deck file path.")?;

    run.deck_path = read_text(reader)?;

    while scan(reader)? {
        let header = read_text(reader)?.to_lowercase();
        if header == "remaining" {
            run.remaining = read_id_list(reader)?;
        } else if header == "working" {
            run.working = read_id_list(reader)?;
        } else if header == "incorrect" {
            run.incorrect = read_id_list(reader)?;
        } else if header == "memorized" {
            run.memorized = read_id_list(reader)?;
        } else {
            return Err(ParsingError::box_new(format!(
                "Unexpected heading {}",
                header
            )));
        }
    }

    Ok(run)
}

fn check_duplicates(run: &PracticeRun) -> Result<HashSet<usize>, Box<dyn Error>> {
    let mut set = HashSet::<usize>::new();

    let id_lists = [&run.remaining, &run.incorrect, &run.memorized, &run.working];

    for v in id_lists {
        for id in v.iter() {
            if !set.insert(*id) {
                return Err(ParsingError::box_new(format!(
                    "Run file invalid: id {} found in multiple locations.",
                    id
                )));
            }
        }
    }

    Ok(set)
}

fn remove_id(run: &mut PracticeRun, id: usize) {
    let id_lists = [
        &mut run.remaining,
        &mut run.incorrect,
        &mut run.memorized,
        &mut run.working,
    ];

    for list in id_lists {
        if let Some(index) = list.iter().position(|_id| *_id == id) {
            list.remove(index);
            return;
        }
    }
}

pub fn load_practice_run(filepath: &str) -> Result<(PracticeRun, Deck), Box<dyn Error>> {
    let mut run = load_practice_run_file(filepath)?;

    let run_ids = check_duplicates(&run)?;

    let deck = load_deck(&run.deck_path)?;

    // Add any newly added cards into the run set.
    for (_, card) in deck.cards.iter() {
        if !run_ids.contains(&card.card_id) {
            run.remaining.push(card.card_id);
        }
    }

    for id in run_ids.iter() {
        if !deck.cards.contains_key(id) {
            remove_id(&mut run, *id);
        }
    }

    Ok((run, deck))
}

fn write_ids(w: &mut impl Write, ids: &[usize]) -> Result<(), Box<dyn Error>> {
    for id in ids.iter() {
        writeln!(w, "{}", id)?;
    }

    Ok(())
}

pub fn save_practice_run(filepath: &str, run: &PracticeRun) -> Result<(), Box<dyn Error>> {
    //Todo: write atomically...
    let file = File::create(filepath)?;
    let mut writer = BufWriter::new(&file);
    let w = &mut writer;

    // Sort to avoid churn in the format of the run file
    let mut remaining = run.remaining.clone();
    remaining.sort();

    let mut working = run.working.clone();
    working.sort();

    let mut incorrect = run.working.clone();
    incorrect.sort();

    let mut memorized = run.working.clone();
    memorized.sort();

    writeln!(w, "")?;

    writeln!(w, "{}", run.deck_path)?;
    writeln!(w, "")?;

    writeln!(w, "remaining")?;
    write_ids(w, &remaining)?;

    writeln!(w, "working")?;
    write_ids(w, &working)?;

    writeln!(w, "incorrect")?;
    write_ids(w, &incorrect)?;

    writeln!(w, "memorized")?;
    write_ids(w, &memorized)?;

    writeln!(w, "")?;

    Ok(())
}
