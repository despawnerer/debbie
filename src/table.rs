use std::iter::FromIterator;

use crate::selection::{Row, Selection};

pub trait Selectable: Clone {
    type Indexer: Indexer<Self>;
}

pub trait Indexer<T>: Default {
    fn add(&mut self, row: Row<T>, item: &T);
    fn remove(&mut self, row: Row<T>, item: &T);
}

#[derive(Clone, Debug)]
pub struct Selector<'table, T: Selectable> {
    pub indexer: &'table T::Indexer,
    selection: Selection<T>,
    table: &'table Table<T>,
}

impl<'table, T> Selector<'table, T>
where
    T: Selectable,
{
    // selecting

    pub fn and(&mut self, selection: &Selection<T>) -> &mut Self {
        self.only(&self.selection & selection)
    }

    pub fn or(&mut self, selection: &Selection<T>) -> &mut Self {
        self.only(&self.selection | selection)
    }

    pub fn none(&mut self) -> &mut Self {
        self.only(Selection::empty())
    }

    pub fn only_row(&mut self, row: Row<T>) -> &mut Self {
        self.only(Selection::of_row(row))
    }

    pub fn only(&mut self, selection: Selection<T>) -> &mut Self {
        self.selection = selection;
        self
    }

    // operations on the selected items

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.table.retrieve_many(self.selection.rows())
    }

    pub fn first(&self) -> Option<T> {
        self.iter().next()
    }

    pub fn collect<B>(&self) -> B
    where
        B: FromIterator<T>,
    {
        self.iter().collect()
    }

    pub fn count(&self) -> u64 {
        self.selection.len()
    }
}

#[derive(Default, Debug)]
pub struct Table<T>
where
    T: Selectable,
{
    items: Vec<T>,
    indexer: T::Indexer,
}

impl<T> Table<T>
where
    T: Selectable,
{
    pub fn in_memory() -> Self {
        Table {
            items: Vec::new(),
            indexer: T::Indexer::default(),
        }
    }

    pub fn select(&self) -> Selector<T> {
        Selector {
            selection: Selection::filled(self.len() as u32),
            indexer: &self.indexer,
            table: self,
        }
    }

    pub fn insert(&mut self, item: T) {
        let row = unsafe { Row::from_index(self.items.len()) };
        self.indexer.add(row, &item);
        self.items.push(item);
    }

    pub fn update<F: Fn(&mut T)>(&mut self, row: Row<T>, update: F) {
        let mut item = self.retrieve(row.clone());
        self.indexer.remove(row.clone(), &item);
        update(&mut item);
        self.indexer.add(row, &item);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn retrieve(&self, row: Row<T>) -> T {
        unsafe { self.items.get_unchecked(row.as_index()).clone() }
    }

    pub fn retrieve_many<'table, I>(&'table self, rows: I) -> impl Iterator<Item = T> + 'table
    where
        I: IntoIterator<Item = Row<T>>,
        I::IntoIter: 'table,
    {
        rows.into_iter().map(move |row| self.retrieve(row))
    }
}
