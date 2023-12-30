use std::{error::Error, fmt::Display};

use rand::{seq::SliceRandom, thread_rng};

use crate::PracticeRun;

#[derive(Debug, Clone, Copy)]
pub enum RunCategory {
    Remaining,
    Memorized,
    Working,
    Incorrect,
}

impl Into<usize> for RunCategory {
    fn into(self) -> usize {
        match self {
            RunCategory::Remaining => 0,
            RunCategory::Memorized => 1,
            RunCategory::Working => 2,
            RunCategory::Incorrect => 3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RunActionError {
    IdNotFound(usize, RunCategory),
}

impl Display for RunActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", &self))
    }
}

impl Error for RunActionError {
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

impl PracticeRun {
    pub fn category_array(&mut self) -> [&mut Vec<usize>; 4] {
        [
            &mut self.remaining,
            &mut self.memorized,
            &mut self.working,
            &mut self.incorrect,
        ]
    }

    fn shuffle_vec(vec: &mut Vec<usize>) {
        vec.shuffle(&mut thread_rng());
    }

    pub fn shuffle(&mut self, category: RunCategory) {
        let array = self.category_array();
        let index: usize = category.into();

        Self::shuffle_vec(array[index])
    }

    pub fn shuffle_all(&mut self) {
        let mut array = self.category_array();
        array.iter_mut().for_each(|v| Self::shuffle_vec(v));
    }

    pub fn move_index(
        &mut self,
        id: usize,
        source_category: RunCategory,
        destination_category: RunCategory,
    ) -> Result<(), RunActionError> {
        let array = self.category_array();
        let src_cat: usize = source_category.into();
        let dest_cat: usize = destination_category.into();

        if let Some(src_index) = array[src_cat].iter().position(|_id| *_id == id) {
            array[src_cat].remove(src_index);
        } else {
            return Err(RunActionError::IdNotFound(id, source_category));
        }

        array[dest_cat].push(id);

        Ok(())
    }

    pub fn move_category(
        &mut self,
        source_category: RunCategory,
        destination_category: RunCategory,
    ) {
        let array = self.category_array();
        let src_cat: usize = source_category.into();
        let dest_cat: usize = destination_category.into();

        let mut src: Vec<usize> = array[src_cat].drain(..).collect();

        array[dest_cat].append(&mut src);
    }

    pub fn skip(&mut self) {
        if let Some(element) = self.remaining.pop() {
            self.remaining.insert(0, element);
        }
    }
}
