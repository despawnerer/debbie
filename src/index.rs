use std::collections::HashMap;
use std::hash::Hash;

use crate::selection::{Row, Selection};

pub trait Index<T> {
    fn add(&mut self, row: Row<T>, item: &T);
    fn remove(&mut self, row: Row<T>, item: &T);
}

// unique

pub struct UniqueIndex<T, V>
where
    V: Eq + Hash,
{
    predicate: fn(&T) -> V,
    rows: HashMap<V, Row<T>>,
}

impl<T, V> UniqueIndex<T, V>
where
    V: Eq + Hash,
{
    pub fn new(predicate: fn(&T) -> V) -> Self {
        Self {
            predicate,
            rows: HashMap::new(),
        }
    }

    pub fn get(&self, value: &V) -> Option<&Row<T>> {
        self.rows.get(value)
    }
}

impl<T, V> Index<T> for UniqueIndex<T, V>
where
    V: Eq + Hash,
{
    fn add(&mut self, row: Row<T>, item: &T) {
        self.rows.insert((self.predicate)(item), row);
    }

    fn remove(&mut self, _row: Row<T>, item: &T) {
        self.rows.remove(&(self.predicate)(item));
    }
}

// discrete

pub struct DiscreteIndex<T, V>
where
    V: Eq + Hash,
{
    predicate: fn(&T) -> &V,
    selections: HashMap<V, Selection<T>>,
    empty: Selection<T>,
}

impl<T, V> DiscreteIndex<T, V>
where
    V: Eq + Hash,
{
    pub fn new(predicate: fn(&T) -> &V) -> Self {
        Self {
            predicate,
            selections: HashMap::new(),
            empty: Selection::empty(),
        }
    }

    pub fn get(&self, value: &V) -> &Selection<T> {
        self.selections.get(value).unwrap_or(&self.empty)
    }
}

impl<T, V> Index<T> for DiscreteIndex<T, V>
where
    V: Eq + Hash + Clone,
{
    fn add(&mut self, row: Row<T>, item: &T) {
        let key = (self.predicate)(item);
        match self.selections.get_mut(key) {
            Some(selection) => selection.add(row),
            None => {
                let mut selection = Selection::empty();
                selection.add(row);
                self.selections.insert((*key).clone(), selection);
            }
        };
    }

    fn remove(&mut self, row: Row<T>, item: &T) {
        let key = (self.predicate)(item);
        if let Some(selection) = self.selections.get_mut(key) {
            selection.remove(row);
        }
    }
}

// boolean

pub struct BooleanIndex<T> {
    predicate: fn(&T) -> bool,
    selection: Selection<T>,
}

impl<T> BooleanIndex<T> {
    pub fn new(predicate: fn(&T) -> bool) -> Self {
        Self {
            predicate,
            selection: Selection::empty(),
        }
    }

    pub fn get(&self) -> &Selection<T> {
        &self.selection
    }
}

impl<T> Index<T> for BooleanIndex<T> {
    fn add(&mut self, row: Row<T>, item: &T) {
        if (self.predicate)(item) {
            self.selection.add(row);
        }
    }

    fn remove(&mut self, row: Row<T>, _item: &T) {
        self.selection.remove(row);
    }
}
