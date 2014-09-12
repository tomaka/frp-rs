use std::{iter, slice};

pub struct StaticIndicesVec<T>(Vec<Option<T>>);

#[deriving(Clone, Show, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Index(uint);

impl<T> StaticIndicesVec<T> {
    pub fn new() -> StaticIndicesVec<T> {
        StaticIndicesVec(Vec::new())
    }

    pub fn push(&mut self, element: T) -> Index {
        let &StaticIndicesVec(ref mut data) = self;

        let index = data.iter().enumerate().find(|&(_, x)| x.is_none()).map(|(k, _)| k);

        match index {
            None => {
                data.push(Some(element));
                Index(data.len() - 1)
            },
            Some(index) => {
                *data.get_mut(index) = Some(element);
                Index(index)
            }
        }
    }

    pub fn remove(&mut self, index: Index) -> T {
        let Index(index) = index;
        let &StaticIndicesVec(ref mut data) = self;

        data.remove(index).unwrap().unwrap()
    }

    pub fn iter<'a>(&'a self) -> iter::FilterMap<'a, &Option<T>, &T, slice::Items<'a, Option<T>>> {
        let &StaticIndicesVec(ref data) = self;
        data.iter().filter_map(|i| i.as_ref())
    }

    pub fn mut_iter<'a>(&'a mut self) -> iter::FilterMap<'a, &'a mut Option<T>, &mut T, slice::MutItems<'a, Option<T>>> {
        let &StaticIndicesVec(ref mut data) = self;
        data.mut_iter().filter_map(|i| i.as_mut())
    }

    pub fn get(&mut self, index: &Index) -> Option<&T> {
        self.0.as_slice().get(index.0).and_then(|v| v.as_ref())
    }

    pub fn get_mut(&mut self, index: &Index) -> Option<&mut T> {
        self.0.as_mut_slice().get_mut(index.0).and_then(|v| v.as_mut())
    }
}
