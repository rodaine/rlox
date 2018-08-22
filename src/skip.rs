#[derive(Default, Debug)]
pub struct SkipList<T: PartialEq> {
    elems: Vec<(usize, T)>,
}

impl<T: PartialEq> SkipList<T> {
    pub fn push(&mut self, idx: usize, el: T) {
        if self.elems.is_empty() {
            self.elems.push((0, el));
            return;
        }

        if let Some((_, val)) = self.elems.last() {
            if *val == el {
                return;
            }
        }

        self.elems.push((idx, el));
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        let mut val: Option<&T> = None;

        for (i, el) in &self.elems {
            match *i {
                x if x == idx => return Some(&el),
                x if x < idx => val = Some(&el),
                _ => return val,
            }
        }

        val
    }
}
