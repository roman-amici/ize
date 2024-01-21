use std::{any, error::Error};

use cursive::{
    align::{Align, HAlign, VAlign},
    event::{EventResult, Key},
    view::{scroll::on_event, Nameable, Resizable},
    views::{
        Button, Dialog, DummyView, EditView, Layer, LinearLayout, NamedView, OnEventView,
        ProgressBar, SelectView, TextArea, TextContent, TextView,
    },
    Cursive, View, reexports::crossbeam_channel::Select, utils::Counter,
};
use ize_core::{
    prelude::{load_deck, load_practice_run, RunCategory},
    Card, Deck, PracticeRun,
};

const MAIN_MENU: &str = "MainMenu";
const RUN_SELECT: &str = "RunSelect";
const CARD_VIEW: &str = "CardView";
const CARD_CONTENT: &str = "CardContent";
const RUN_PROGRESS_BAR: &str = "RunProgress";

struct RunState {
    run_data: RunData,
    card_content_state: CardContentState,
    count: usize,
}

struct RunData {
    run: PracticeRun,
    deck: Deck,
}

struct DeckPath(String);

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

fn main() {
    let mut siv = cursive::default();

    main_menu(&mut siv);

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

fn new_run(siv: &mut Cursive) {
    let run_state = new_run_state("../test_deck/state_capitols.deck");

    match run_state {
        Err(e) => {
            show_error(siv, e.as_ref());
        }
        Ok(run_state) => {
            siv.set_user_data(run_state);
            begin_run(siv);
        }
    }
}

fn new_run_state(deck_path: &str) -> Result<RunState, Box<dyn Error>> {
    let deck = load_deck(deck_path)?;

    let run = PracticeRun::new_from_deck(&deck);
    let run_data = RunData { deck, run };

    Ok(RunState {
        card_content_state: CardContentState::Front,
        run_data,
        count: 0,
    })
}

fn card_choice(siv: &mut Cursive, destination: RunCategory) {
    siv.with_user_data(|state: &mut RunState| match destination {
        RunCategory::Remaining => state.run_data.run.skip(),
        _ => {
            state
                .run_data
                .run
                .move_last(RunCategory::Remaining, destination)
                .expect("Error");
            state.count += 1
        }
    });

    let count = siv.user_data::<RunState>().unwrap().count;

    siv.call_on_name(RUN_PROGRESS_BAR, |view: &mut ProgressBar| {
        view.set_value(count)
    })
    .expect("View not found");

    show_current_card(siv);
}

fn reset_run(siv : &mut Cursive)
{
    let count = siv.user_data::<RunState>().unwrap().count;
    let max = siv.user_data::<RunState>().unwrap().run_data.run.remaining.len();
    siv.call_on_name(RUN_PROGRESS_BAR, |view: &mut ProgressBar| {
        view.set_max(max);
        view.set_value(count);
    })
    .expect("View not found");

    show_current_card(siv);
}

fn flip_card(siv: &mut Cursive) {
    let state: CardContentState = siv.user_data::<RunState>().unwrap().card_content_state;

    match state {
        CardContentState::Front => {
            set_card_back(siv);
        }
        CardContentState::Back => {
            set_card_front(siv);
        }
    };
}

fn set_card_back(siv: &mut Cursive) {
    siv.call_on_name(CARD_VIEW, |view: &mut Dialog| {
        view.set_title("Back");
    })
    .expect("View not found");

    let content = siv
        .with_user_data(|state: &mut RunState| {
            state.card_content_state = CardContentState::Back;
            state.run_data.current_card_back().unwrap()
        })
        .expect("Expected run data");

    siv.call_on_name(CARD_CONTENT, |view: &mut TextView| {
        view.set_content(content);
    })
    .expect("View not found");
}

fn set_card_front(siv: &mut Cursive) {
    siv.call_on_name(CARD_VIEW, |view: &mut Dialog| {
        view.set_title("Front");
    })
    .expect("View not found");

    let content = siv
        .with_user_data(|state: &mut RunState| {
            state.card_content_state = CardContentState::Front;
            state.run_data.current_card_front().unwrap()
        })
        .expect("Expected run data");

    siv.call_on_name(CARD_CONTENT, |view: &mut TextView| {
        view.set_content(content);
    })
    .expect("View not found");
}

fn show_done_menu(siv : &mut Cursive)
{
    let (mem_count, working_count, inc_count) = siv.with_user_data(|state : &mut RunState|
    {
        (state.run_data.run.memorized.len(), state.run_data.run.working.len(), state.run_data.run.incorrect.len())
    }).expect("User data failed.");
    let all_count = mem_count + working_count + inc_count;

    let mut select_view = SelectView::new();
    select_view.add_item(format!("All ({})", all_count), RunCategory::Remaining);
    if inc_count > 0 {
        select_view.add_item(format!("Incorrect ({})",inc_count), RunCategory::Incorrect);
    }
    if working_count > 0 {
        select_view.add_item(format!("Working ({})", working_count), RunCategory::Working);
    } 
    if mem_count > 0 {
        select_view.add_item(format!("Memorized ({})", mem_count), RunCategory::Memorized);
    }
    
    let select_view = select_view.on_submit(|s, run_category| {
        s.with_user_data(|state : &mut RunState| {
            match run_category {
                RunCategory::Remaining => state.run_data.run.reset(),
                _ => {
                    state.run_data.run.move_category(*run_category, RunCategory::Remaining);
                    state.run_data.run.shuffle(RunCategory::Remaining);
                }
            }
            state.count = 0;
        }).expect("Run data not found");

        s.pop_layer();
        reset_run(s);

    });

    siv.add_layer(
        Dialog::new()
            .title("Which pile to reshuffle?")
            .content(select_view)
            .button("Quit", |s| {
                s.pop_layer();
                main_menu(s);
            }),
    )
}

fn show_current_card(siv: &mut Cursive) {
    let done = siv
        .user_data::<RunState>()
        .as_ref()
        .unwrap()
        .run_data
        .is_done();

    if done {
        show_done_menu(siv);
        return;
    }

    set_card_front(siv);
}

// Assume that user data has been set
fn begin_run(siv: &mut Cursive) {
    siv.pop_layer();

    let card_view = Dialog::new()
        .title("Front")
        .content(
            TextView::new("Placeholder")
                .align(Align {
                    h: HAlign::Center,
                    v: VAlign::Center,
                })
                .with_name(CARD_CONTENT),
        )
        .with_name(CARD_VIEW)
        .fixed_height(15);

    let bottom_menu = LinearLayout::horizontal()
        .child(Button::new("1.Skip", |s| {
            card_choice(s, RunCategory::Remaining);
        }))
        .child(Button::new("2.Incorrect", |s| {
            card_choice(s, RunCategory::Incorrect);
        }))
        .child(Button::new("3.Working", |s| {
            card_choice(s, RunCategory::Working);
        }))
        .child(Button::new("4.Memorized", |s| {
            card_choice(s, RunCategory::Memorized);
        }))
        .child(DummyView)
        .child(Button::new("Quit", |s| {
            main_menu(s);
        }));

    let n_cards = siv
        .user_data::<RunState>()
        .unwrap()
        .run_data
        .run
        .remaining
        .len();

    let progress = ProgressBar::new()
        .min(0)
        .max(n_cards)
        .with_label(|value, (_, max)| format!("{value} / {max}"))
        .with_name(RUN_PROGRESS_BAR);

    let run_screen = LinearLayout::vertical()
        .child(card_view)
        .child(progress)
        .child(bottom_menu);

    let key_wrapper = OnEventView::new(run_screen)
        .on_event(' ', |s| flip_card(s))
        .on_event('1', |s| card_choice(s, RunCategory::Remaining))
        .on_event('2', |s| card_choice(s, RunCategory::Incorrect))
        .on_event('3', |s| card_choice(s, RunCategory::Working))
        .on_event('4', |s| card_choice(s, RunCategory::Memorized))
        .on_event('q', |s| {
            main_menu(s);
        });

    siv.add_layer(key_wrapper);
    show_current_card(siv);
}

fn show_error(siv: &mut Cursive, err: &dyn Error) {
    siv.add_layer(
        Dialog::new()
            .title("Error!")
            .content(TextView::new(err.to_string()))
            .button("Ok", |s| {
                s.pop_layer();
            }),
    )
}

fn load_run(siv: &mut Cursive, file_path: &str) {
    match load_practice_run(file_path) {
        Ok((run, deck)) => {
            siv.set_user_data(RunData { deck, run });
            begin_run(siv);
        }
        Err(e) => show_error(siv, e.as_ref()),
    }
}

fn resume_run(siv: &mut Cursive) {
    siv.pop_layer();

    siv.add_layer(
        Dialog::new()
            .title("Enter")
            .content(
                EditView::new()
                    .on_submit(load_run)
                    .with_name(RUN_SELECT)
                    .fixed_width(50),
            )
            .button("Load", |s| {
                let file_path = s
                    .call_on_name(RUN_SELECT, |view: &mut EditView| view.get_content())
                    .unwrap();

                load_run(s, &file_path);
            })
            .button("Back", main_menu),
    )
}
