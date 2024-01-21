use std::error::Error;

use cursive::{
    views::{Dialog, TextView},
    Cursive,
};

pub fn show_error(siv: &mut Cursive, err: &dyn Error) {
    siv.add_layer(
        Dialog::new()
            .title("Error!")
            .content(TextView::new(err.to_string()))
            .button("Ok", |s| {
                s.pop_layer();
            }),
    )
}
