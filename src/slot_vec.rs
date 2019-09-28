use std::ops::{ Index, IndexMut };

#[derive(Default)]
pub struct SlotVec<T> {
    vec: Vec<Option<T>>,
}

impl<T> SlotVec<T> {
    pub fn new() -> Self {
        SlotVec { vec: Vec::new() }
    }

    pub fn push(&mut self, val: T) -> usize {
        let index = self.vec.len();
        self.vec.push(Some(val));
        index
    }

    pub fn new_slot(&mut self) -> usize {
        let slot = self.vec.len();
        self.vec.push(None);
        slot
    }

    pub fn fill_slot(&mut self, slot: usize, val: T) {
        if self.vec[slot].is_some() {
            panic!("Slot {} is already occupied", slot)
        }
        self.vec[slot] = Some(val);
    }

    pub fn build(self) -> Box<[T]> {
        self.vec.into_iter().map(|v|v.unwrap()).collect::<Vec<T>>().into_boxed_slice()
    }
}

impl<T> Index<usize> for SlotVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.vec[index].as_ref().unwrap()
    }
}

impl<T> IndexMut<usize> for SlotVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.vec[index].as_mut().unwrap()
    }
}
