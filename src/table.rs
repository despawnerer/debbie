use std::iter::FromIterator;

use crate::selection::{Row, Selection};

pub trait Selectable: Clone {
    type Indexer: Indexer<Self>;
}

pub trait Indexer<T> {
    fn new() -> Self;
    fn add(&mut self, row: Row<T>, item: &T);
    fn remove(&mut self, row: Row<T>, item: &T);
}

pub struct EmptyIndexer;

impl<T> Indexer<T> for EmptyIndexer {
    fn new() -> Self {
        Self
    }
    fn add(&mut self, _: Row<T>, _: &T) {}
    fn remove(&mut self, _: Row<T>, _: &T) {}
}

#[derive(Clone, Debug)]
pub struct Query<T, X>
where
    T: Selectable,
    X: AsRef<Table<T>>,
{
    selection: Selection<T>,
    table: X,
}

impl<T, X> Query<T, X>
where
    T: Selectable,
    X: AsRef<Table<T>>,
{
    pub fn indexer(&self) -> &T::Indexer {
        &self.table.as_ref().indexer
    }

    // selecting

    pub fn and(&mut self, selection: &Selection<T>) -> &mut Self {
        self.selection &= selection;
        self
    }

    pub fn or(&mut self, selection: &Selection<T>) -> &mut Self {
        self.selection |= selection;
        self
    }

    pub fn none(&mut self) -> &mut Self {
        self.selection = Selection::empty();
        self
    }

    pub fn only_row(&mut self, row: Row<T>) -> &mut Self {
        self.selection = Selection::of_row(row);
        self
    }

    pub fn maybe_only_row(&mut self, maybe_row: Option<Row<T>>) -> &mut Self {
        match maybe_row {
            Some(row) => self.only_row(row),
            None => self.none(),
        }
    }

    pub fn only(&mut self, selection: Selection<T>) -> &mut Self {
        self.selection = selection;
        self
    }

    // operations on the selected items

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.table.as_ref().retrieve_many(self.selection.rows())
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

impl<T, X> Query<T, X>
where
    T: Selectable,
    X: AsRef<Table<T>> + AsMut<Table<T>>,
{
    pub fn apply<F: Fn(&mut T) + Clone>(&mut self, update: F) {
        for row in self.selection.rows() {
            self.table.as_mut().update_row(row, update.clone())
        }
    }
}

#[derive(Debug)]
pub struct Table<T>
where
    T: Selectable,
{
    items: Vec<T>,
    indexer: T::Indexer,
}

impl<T> AsRef<Table<T>> for Table<T>
where
    T: Selectable,
{
    fn as_ref(&self) -> &Table<T> {
        self
    }
}

impl<T> AsMut<Table<T>> for Table<T>
where
    T: Selectable,
{
    fn as_mut(&mut self) -> &mut Table<T> {
        self
    }
}

impl<T> Table<T>
where
    T: Selectable,
{
    pub fn in_memory() -> Self {
        Table {
            items: Vec::new(),
            indexer: T::Indexer::new(),
        }
    }

    pub fn select(&self) -> Query<T, &Table<T>> {
        Query {
            selection: Selection::filled(self.len() as u32),
            table: self,
        }
    }

    pub fn insert(&mut self, item: T) {
        let row = unsafe { Row::from_index(self.items.len()) };
        self.indexer.add(row, &item);
        self.items.push(item);
    }

    pub fn update(&mut self) -> Query<T, &mut Table<T>> {
        Query {
            selection: Selection::filled(self.len() as u32),
            table: self,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn update_row<F: Fn(&mut T)>(&mut self, row: Row<T>, update: F) {
        let mut item = unsafe { self.items.get_unchecked_mut(row.as_index()) };
        self.indexer.remove(row, &item);
        update(&mut item);
        self.indexer.add(row, &item);
    }

    fn retrieve_row(&self, row: Row<T>) -> T {
        unsafe { self.items.get_unchecked(row.as_index()).clone() }
    }

    fn retrieve_many<'table, I>(&'table self, rows: I) -> impl Iterator<Item = T> + 'table
    where
        I: IntoIterator<Item = Row<T>>,
        I::IntoIter: 'table,
    {
        rows.into_iter().map(move |row| self.retrieve_row(row))
    }
}
