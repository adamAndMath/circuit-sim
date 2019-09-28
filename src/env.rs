use std::collections::HashMap;
use std::borrow::Borrow;
use std::ops::Index;
use std::hash::Hash;
use std::iter::{ Extend, FromIterator };
use std::fmt::Debug;

#[derive(Debug)]
pub struct Env<T> {
    map: HashMap<String, T>,
}

impl<T> Default for Env<T> {
    fn default() -> Self {
        Env { map: HashMap::default() }
    }
}

impl<T> Env<T> {
    pub fn insert(&mut self, key: String, val: T) {
        if self.map.insert(key.clone(), val).is_some() {
            panic!("Duplicate key: {}", key);
        }
    }
}

impl<T, I: ?Sized + Hash + Eq + Debug> Index<&I> for Env<T> where String: Borrow<I> {
    type Output = T;
    fn index(&self, i: &I) -> &T {
        &self.map.get(i).unwrap_or_else(||panic!("Unknown key: {:?}", i))
    }
}

impl<T> Extend<(String, T)> for Env<T> {
    fn extend<I: IntoIterator<Item = (String, T)>>(&mut self, iter: I) {
        for (key, val) in iter {
            self.insert(key, val);
        }
    }
}

impl<T> FromIterator<(String, T)> for Env<T> {
    fn from_iter<I: IntoIterator<Item = (String, T)>>(iter: I) -> Self {
        let mut env = Env { map: HashMap::new() };
        env.extend(iter);
        env
    }
}
