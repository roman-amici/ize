use std::error::Error;

use cursive::{
    align::{Align, HAlign, VAlign},
    event::{EventResult, Key},
    view::{Nameable, Resizable},
    views::{
        Button, Dialog, DummyView, EditView, Layer, LinearLayout, NamedView, OnEventView,
        SelectView, TextArea, TextContent, TextView,
    },
    Cursive, View,
};
use ize_core::{
    prelude::{load_practice_run, RunCategory},
    Deck, PracticeRun,
};

const MAIN_MENU: &str = "MainMenu";
const RUN_SELECT: &str = "RunSelect";
const CARD_VIEW: &str = "CardView";
const CARD_CONTENT: &str = "CardContent";

struct RunData {
    run: PracticeRun,
    deck: Deck,
}

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
    begin_run(siv);
}

fn card_choice(siv: &mut Cursive, destination: RunCategory) {
    println!("{:?}", destination);
}

fn flip_card(siv: &mut Cursive) {
    let state: CardContentState = siv.take_user_data().expect("Expected CardContentState");

    match state {
        CardContentState::Front => {
            siv.call_on_name(CARD_VIEW, |view: &mut Dialog| {
                view.set_title("Back");
            })
            .expect("View not found");

            siv.call_on_name(CARD_CONTENT, |view: &mut TextView| {
                view.set_content("Card content back");
            })
            .expect("View not found");

            siv.set_user_data(CardContentState::Back);
        }
        CardContentState::Back => {
            siv.call_on_name(CARD_VIEW, |view: &mut Dialog| {
                view.set_title("Front");
            })
            .expect("View not found");

            siv.call_on_name(CARD_CONTENT, |view: &mut TextView| {
                view.set_content("Card content front");
            })
            .expect("View not found");

            siv.set_user_data(CardContentState::Front);
        }
    }
}

// Assume that user data has been set
fn begin_run(siv: &mut Cursive) {
    siv.pop_layer();

    siv.set_user_data(CardContentState::Front);

    let card_view = Dialog::new()
        .title("Front")
        .content(
            TextView::new("Card Content Front")
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
        .child(Button::new("Quit", |s| s.quit()));

    let wrapper = OnEventView::new(LinearLayout::vertical().child(card_view).child(bottom_menu))
        .on_event(' ', |s| flip_card(s))
        .on_event('1', |s| card_choice(s, RunCategory::Remaining));

    siv.add_layer(wrapper);
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
