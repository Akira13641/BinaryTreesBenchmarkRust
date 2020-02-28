// Original author: Akira1364
// Year: 2020
// License: MIT

#![allow(incomplete_features)]
#![feature(alloc_layout_extra)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(core_intrinsics)]

use std::{alloc::*, intrinsics::*, ptr::*};

struct TNonFreePooledMemManager<T, const N: usize> {
  cur_size: usize,
  cur_item: *mut T,
  end_item: *mut T,
  items: Vec<(*mut T, usize)>,
}

impl<T, const N: usize> TNonFreePooledMemManager<T, N> {
  #[inline(always)]
  const fn new() -> Self {
    Self {
      cur_size: size_of::<T>() * N,
      cur_item: null_mut(),
      end_item: null_mut(),
      items: Vec::new(),
    }
  }

  #[inline]
  fn clear(&mut self) {
    let length = self.items.len();
    if length > 0 {
      for i in 0..length {
        unsafe {
          let tup = self.items.get_unchecked(i);
          dealloc(tup.0 as *mut u8, Layout::array::<T>(tup.1).unwrap());
        }
      }
      self.items.truncate(0);
      self.cur_size = size_of::<T>() * N;
      self.cur_item = null_mut();
      self.end_item = null_mut();
    }
  }

  #[inline]
  fn new_item(&mut self) -> &mut T {
    if self.cur_item == self.end_item {
      self.cur_size += self.cur_size;
      let num_items = self.cur_size / size_of::<T>();
      self.cur_item =
        unsafe { alloc_zeroed(Layout::array::<T>(num_items).unwrap()) as *mut T };
      self.items.push((self.cur_item, num_items));
      self.end_item = self.cur_item;
      self.end_item = unsafe { (self.end_item as *mut u8).add(self.cur_size) as *mut T };
    }
    let result = self.cur_item;
    unsafe {
      self.cur_item = self.cur_item.offset(1);
      &mut *result
    }
  }

  #[inline]
  fn enumerate_items<F>(&mut self, mut f: F)
  where
    F: FnMut(&mut T) -> (),
  {
    let length = self.items.len();
    if length > 0 {
      let mut size = size_of::<T>() * N;
      for i in 0..length {
        size += size;
        unsafe {
          let mut p = self.items.get_unchecked_mut(i).0;
          let mut last = p;
          last = (last as *mut u8).add(size) as *mut T;
          if i == length - 1 {
            last = self.end_item;
          }
          while p != last {
            f(&mut *p);
            p = p.offset(1);
          }
        }
      }
    }
  }
}

impl<T, const N: usize> Drop for TNonFreePooledMemManager<T, N> {
  #[inline(always)]
  fn drop(&mut self) {
    self.clear();
  }
}
