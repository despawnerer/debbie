use std::iter::FromIterator;
use std::marker::{Copy, PhantomData};
use std::ops::{BitAnd, BitOr, BitAndAssign, BitOrAssign};

use croaring::Bitmap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row<T> {
    value: u32,
    _marker: PhantomData<T>,
}

impl<T> Copy for Row<T> {}

impl<T> Clone for Row<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Row<T> {
    pub unsafe fn from_index(value: usize) -> Self {
        Row {
            value: value as u32,
            _marker: PhantomData,
        }
    }

    pub fn as_index(self) -> usize {
        self.value as usize
    }

    fn from_u32(value: u32) -> Self {
        Row {
            value,
            _marker: PhantomData,
        }
    }

    fn as_u32(self) -> u32 {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct Selection<T> {
    bitmap: Bitmap,
    _marker: PhantomData<T>,
}

impl<T> Selection<T> {
    pub fn empty() -> Self {
        Selection {
            bitmap: Bitmap::create(),
            _marker: PhantomData,
        }
    }

    pub fn filled(count: u32) -> Self {
        Selection {
            bitmap: (0..count).collect(),
            _marker: PhantomData,
        }
    }

    pub fn of_row(row: Row<T>) -> Self {
        let mut selection = Self::empty();
        selection.add(row);
        selection
    }

    fn from_bitmap(bitmap: Bitmap) -> Self {
        Selection {
            bitmap,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> u64 {
        self.bitmap.cardinality()
    }

    pub fn is_empty(&self) -> bool {
        self.bitmap.is_empty()
    }

    pub fn rows(&self) -> impl Iterator<Item = Row<T>> + '_ {
        self.bitmap.iter().map(Row::from_u32)
    }

    pub fn add(&mut self, row: Row<T>) {
        self.bitmap.add(row.as_u32());
    }

    pub fn remove(&mut self, row: Row<T>) {
        self.bitmap.remove(row.as_u32());
    }
}

impl<T> Default for Selection<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> BitAnd for &Selection<T> {
    type Output = Selection<T>;

    fn bitand(self, rhs: Self) -> Self::Output {
        Selection::from_bitmap(&self.bitmap & &rhs.bitmap)
    }
}

impl<T> BitOr for &Selection<T> {
    type Output = Selection<T>;

    fn bitor(self, rhs: Self) -> Self::Output {
        Selection::from_bitmap(&self.bitmap | &rhs.bitmap)
    }
}

impl<T> BitAndAssign<&Selection<T>> for Selection<T> {
    fn bitand_assign(&mut self, rhs: &Selection<T>) {
        self.bitmap.and_inplace(&rhs.bitmap);
    }
}

impl<T> BitOrAssign<&Selection<T>> for Selection<T> {
    fn bitor_assign(&mut self, rhs: &Selection<T>) {
        self.bitmap.or_inplace(&rhs.bitmap);
    }
}

impl<T> FromIterator<Row<T>> for Selection<T> {
    fn from_iter<I: IntoIterator<Item = Row<T>>>(iter: I) -> Self {
        Selection::from_bitmap(iter.into_iter().map(|row| row.as_u32()).collect())
    }
}
