use std::{cmp::max, path::Path};

use cursive::{
    align::{Align, HAlign, VAlign},
    view::{Nameable, Resizable},
    views::{
        Button, Dialog, DummyView, EditView, LinearLayout, OnEventView, ProgressBar, SelectView,
        TextView,
    },
    Cursive,
};
use ize_core::prelude::*;

use crate::{
    file_explorer::show_file_explorer, main_menu, utils::show_error, CardContentState, RunData,
    RunState, RUN_SELECT,
};

const CARD_VIEW: &str = "CardView";
const CARD_CONTENT: &str = "CardContent";
const RUN_PROGRESS_BAR: &str = "RunProgress";

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

fn reset_run(siv: &mut Cursive) {
    let count = siv.user_data::<RunState>().unwrap().count;
    let max = siv
        .user_data::<RunState>()
        .unwrap()
        .run_data
        .run
        .remaining
        .len();
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

fn show_done_menu(siv: &mut Cursive) {
    let (mem_count, working_count, inc_count) = siv
        .with_user_data(|state: &mut RunState| {
            (
                state.run_data.run.memorized.len(),
                state.run_data.run.working.len(),
                state.run_data.run.incorrect.len(),
            )
        })
        .expect("User data failed.");
    let all_count = mem_count + working_count + inc_count;

    let mut select_view = SelectView::new();
    select_view.add_item(format!("All ({})", all_count), RunCategory::Remaining);
    if inc_count > 0 {
        select_view.add_item(format!("Incorrect ({})", inc_count), RunCategory::Incorrect);
    }
    if working_count > 0 {
        select_view.add_item(format!("Working ({})", working_count), RunCategory::Working);
    }
    if mem_count > 0 {
        select_view.add_item(format!("Memorized ({})", mem_count), RunCategory::Memorized);
    }

    let select_view = select_view.on_submit(|s, run_category| {
        s.with_user_data(|state: &mut RunState| {
            match run_category {
                RunCategory::Remaining => state.run_data.run.reset(),
                _ => {
                    state
                        .run_data
                        .run
                        .move_category(*run_category, RunCategory::Remaining);
                    state.run_data.run.shuffle(RunCategory::Remaining);
                }
            }
            state.count = 0;
        })
        .expect("Run data not found");

        s.pop_layer();
        reset_run(s);
    });

    siv.add_layer(
        Dialog::new()
            .title("Which pile to reshuffle?")
            .content(select_view)
            .button("Quit", |s| {
                save_or_quit(s);
            }),
    )
}

fn save_and_quit(siv: &mut Cursive) {
    let run_state: RunState = siv.take_user_data().unwrap();

    let base_path = Path::new(&run_state.run_data.run.deck_path)
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    show_file_explorer(
        siv,
        base_path,
        Box::new(move |s, file_path| {
            let result = save_practice_run(file_path, &run_state.run_data.run);

            match result {
                Ok(_) => {
                    s.pop_layer();
                    main_menu(s);
                }
                Err(e) => {
                    show_error(s, e.as_ref());
                }
            }
        }),
        Box::new(|s| {
            s.pop_layer();
            main_menu(s);
        }),
    )
}

fn save_or_quit(siv: &mut Cursive) {
    siv.pop_layer();
    siv.add_layer(
        Dialog::new()
            .title("Save this run?")
            .button("Save", |s| {
                s.pop_layer();
                save_and_quit(s);
            })
            .button("Don't save", |s| {
                s.pop_layer();
                main_menu(s);
            }),
    );
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
pub fn begin_run(siv: &mut Cursive) {
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
            save_or_quit(s);
        }));

    let n_cards = siv
        .user_data::<RunState>()
        .unwrap()
        .run_data
        .run
        .remaining
        .len();

    // if max = 0, then the progress bar will panic.
    let max_cards = max(1, n_cards);
    let progress = ProgressBar::new()
        .min(0)
        .max(max_cards)
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
