use std::ops::{Index, IndexMut};

use super::dir::Dir;

pub struct DirArray {
    dirs: Vec<Dir>,
}

impl DirArray {
    pub fn new(start_dir: Dir) -> DirArray {
        let mut dirs = Vec::new();
        dirs.push(start_dir);
        DirArray { dirs }
    }

    pub fn push(&mut self, dir: Dir) {
        self.dirs.push(dir);
    }

    pub fn pop(&mut self) -> Option<Dir> {
        self.dirs.pop()
    }

    pub fn current(&mut self) -> Option<&mut Dir> {
        self.dirs.last_mut()
    }
}

impl Index<usize> for DirArray {
    type Output = Dir;

    fn index(&self, index: usize) -> &Self::Output {
        &self.dirs[index]
    }
}

impl IndexMut<usize> for DirArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.dirs[index]
    }
}
