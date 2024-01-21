use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use cursive::{
    view::{Nameable, Resizable},
    views::{LinearLayout, Panel, SelectView, TextArea, TextContent, TextView},
    Cursive, View,
};

use crate::utils::show_error;

struct FileState {
    current_directory: String,
}

const FILE_EXPLORER_SELECT_LIST: &str = "FILE_EXPLORER_SELECT_LIST";
const FILE_EXPLORER_CURRENT_DIRECTORY: &str = "FILE_EXPLORER_CURRENT_DIRECTORY";
const FILE_EXPLORER_FILE_NAME: &str = "FILE_EXPLORER_FILE_NAME";

fn update_file_name(siv: &mut Cursive, file_name: String) {
    siv.call_on_name(FILE_EXPLORER_FILE_NAME, |view: &mut TextArea| {
        view.set_content(file_name);
    });
}

pub fn show_file_explorer(siv: &mut Cursive, base_path: String) {
    let base_path = Path::new(&base_path)
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let fs = Arc::new(Mutex::new(FileState {
        current_directory: base_path.clone(),
    }));

    let fs1 = fs.clone();

    let sl = SelectView::<String>::new()
        .on_submit(move |s, file: &String| {
            let mut update_directory = false;
            {
                let mut file_state = fs1.lock().unwrap();
                let dir = Path::new(&file_state.current_directory);

                if file == ".." {
                    if let Some(parent) = dir.parent() {
                        file_state.current_directory = parent.to_str().unwrap().to_string();
                        update_directory = true;
                    }
                } else if file == "." {
                    update_file_name(s, dir.as_os_str().to_str().unwrap().to_string())
                } else {
                    let abs_path = dir.join(Path::new(file));
                    if abs_path.is_dir() {
                        file_state.current_directory = abs_path.to_str().unwrap().to_string();
                        update_directory = true;
                    } else {
                        update_file_name(s, abs_path.to_str().unwrap().to_string());
                    }
                }
            };

            if update_directory {
                update_directory_view(s, fs1.clone());
            }
        })
        .with_name(FILE_EXPLORER_SELECT_LIST)
        .fixed_size((50, 15));

    let layout = LinearLayout::vertical()
        .child(TextView::new(base_path.clone()).with_name(FILE_EXPLORER_CURRENT_DIRECTORY))
        .child(sl)
        .child(TextArea::new().with_name(FILE_EXPLORER_FILE_NAME))
        .fixed_size((55, 20));

    siv.add_layer(layout);

    update_directory_view(siv, fs);
}

fn update_directory_view(siv: &mut Cursive, fs: Arc<Mutex<FileState>>) {
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
