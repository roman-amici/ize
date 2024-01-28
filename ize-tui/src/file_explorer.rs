use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use cursive::{
    view::{Nameable, Resizable, Scrollable},
    views::{Dialog, EditView, LinearLayout, SelectView, TextView},
    Cursive,
};

use crate::utils::show_error;

struct FileState {
    current_directory: String,
    file_action: Box<dyn Fn(&mut Cursive, &str)>,
}

const FILE_EXPLORER_SELECT_LIST: &str = "FILE_EXPLORER_SELECT_LIST";
const FILE_EXPLORER_CURRENT_DIRECTORY: &str = "FILE_EXPLORER_CURRENT_DIRECTORY";
const FILE_EXPLORER_FILE_NAME: &str = "FILE_EXPLORER_FILE_NAME";

fn update_file_name(siv: &mut Cursive, file_name: String) -> bool {
    siv.call_on_name(FILE_EXPLORER_FILE_NAME, |view: &mut EditView| {
        let equal = view.get_content().as_ref() == &file_name;

        view.set_content(file_name);

        equal
    })
    .unwrap()
}

fn on_file_selector_submit(s: &mut Cursive, file: &String, file_state: &Mutex<FileState>) {
    let mut update_directory = false;
    {
        let mut file_state = file_state.lock().unwrap();
        let dir = Path::new(&file_state.current_directory);

        if file == ".." {
            if let Some(parent) = dir.parent() {
                file_state.current_directory = parent.to_str().unwrap().to_string();
                update_directory = true;
            }
        } else if file == "." {
            update_file_name(s, dir.as_os_str().to_str().unwrap().to_string());
        } else {
            let abs_path = dir.join(Path::new(file));
            if abs_path.is_dir() {
                file_state.current_directory = abs_path.to_str().unwrap().to_string();
                update_directory = true;
            } else {
                let equal = update_file_name(s, abs_path.to_str().unwrap().to_string());

                if equal {
                    let file_name = read_selected_file(s);
                    (file_state.file_action)(s, &file_name);
                }
            }
        }
    };

    if update_directory {
        update_directory_view(s, file_state);
    }
}

pub fn show_file_explorer(
    siv: &mut Cursive,
    base_path: String,
    select_action: Box<dyn Fn(&mut Cursive, &str)>,
    cancel_action: Box<dyn Fn(&mut Cursive)>,
) {
    let base_path = Path::new(&base_path)
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let fs = Arc::new(Mutex::new(FileState {
        current_directory: base_path.clone(),
        file_action: select_action,
    }));

    let fs1 = fs.clone();
    let fs2 = fs.clone();

    let sl = SelectView::<String>::new()
        .on_submit(move |s, file: &String| on_file_selector_submit(s, file, &fs1))
        .with_name(FILE_EXPLORER_SELECT_LIST)
        .scrollable()
        .fixed_size((50, 13));

    let layout = LinearLayout::vertical()
        .child(TextView::new(base_path.clone()).with_name(FILE_EXPLORER_CURRENT_DIRECTORY))
        .child(sl)
        .child(EditView::new().with_name(FILE_EXPLORER_FILE_NAME))
        .fixed_size((55, 20));

    siv.add_layer(
        Dialog::new()
            .title("Select a file")
            .content(layout)
            .button("Cancel", move |s| {
                // Close the dialog
                cancel_action(s);
            })
            .button("Ok", move |s| {
                let file_name = read_selected_file(s);
                {
                    let file_data = fs2.lock().unwrap();
                    (file_data.file_action)(s, &file_name);
                }
            }),
    );

    update_directory_view(siv, &fs);
}

fn read_selected_file(s: &mut Cursive) -> String {
    s.call_on_name(FILE_EXPLORER_FILE_NAME, |view: &mut EditView| {
        view.get_content().to_string()
    })
    .expect("Expected view.")
}

fn update_directory_view(siv: &mut Cursive, fs: &Mutex<FileState>) {
    let path = {
        let file_state = fs.lock().unwrap();
        PathBuf::from_str(&file_state.current_directory).unwrap()
    };
    let path2 = path.clone().to_str().unwrap().to_string();

    let result = siv
        .call_on_name(
            FILE_EXPLORER_SELECT_LIST,
            move |view: &mut SelectView<String>| -> Result<(), Box<dyn Error>> {
                let choices = read_choices(path)?;

                view.clear();
                view.add_all(choices.into_iter().map(|x| (x.clone(), x)));
                Ok(())
            },
        )
        .unwrap();

    if let Err(err) = result {
        show_error(siv, &err.as_ref());
        return;
    }

    siv.call_on_name(
        FILE_EXPLORER_CURRENT_DIRECTORY,
        move |view: &mut TextView| view.set_content(path2),
    )
    .unwrap();
}

fn read_choices(path: PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let paths = fs::read_dir(path)?;

    let mut file_names = vec![".".to_string(), "..".to_string()];
    for path in paths {
        if let Ok(path) = path {
            file_names.push(path.file_name().to_str().unwrap().to_string());
        }
    }

    Ok(file_names)
}
