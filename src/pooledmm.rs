// Original author: Akira1364
// Year: 2020
// License: MIT

use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
use std::mem::{align_of, size_of};
use std::ptr::null_mut;

// Using the allocator API is the closest you can get to something like this in Rust without doing
// anything that would just blatantly be undefined behavior. That said, having to track the size of
// the allocations ourselves only seems to slow things down slightly compared to the other versions.
pub struct TNonFreePooledMemManager<T, const INIT_SIZE: usize> {
  cur_size: usize,
  cur_item: *mut T,
  end_item: *mut T,
  items: Vec<(*mut T, Layout)>,
}

impl<T, const INIT_SIZE: usize> TNonFreePooledMemManager<T, INIT_SIZE> {
  #[inline(always)]
  pub const fn new() -> Self {
    Self {
      cur_size: INIT_SIZE,
      cur_item: null_mut(),
      end_item: null_mut(),
      items: Vec::new(),
    }
  }

  #[inline]
  pub fn clear(&mut self) {
    if !self.items.is_empty() {
      for tup in &self.items {
        unsafe {
          dealloc(tup.0 as *mut u8, tup.1);
        }
      }
      self.items.clear();
      self.cur_size = INIT_SIZE;
      self.cur_item = null_mut();
      self.end_item = null_mut();
    }
  }

  #[inline]
  pub unsafe fn new_item(&mut self) -> *mut T {
    if self.cur_item == self.end_item {
      self.cur_size += self.cur_size;
      let layout = Layout::<T>::from_size_align(
        size_of::<T>() * self.cur_size, align_of::<T>()
      ).unwrap();
      self.cur_item = alloc_zeroed(layout) as *mut T;
      // Generally I feel like if `cur_item` is actually null the user probably has bigger issues to
      // deal with, but properly checking for it doesn't make things noticeably slower so there's no
      // real reason not to.
      if self.cur_item.is_null() {
        handle_alloc_error(layout)
      } else {
        self.items.push((self.cur_item, layout));
      }
      self.end_item = self.cur_item;
      self.end_item = self.end_item.add(self.cur_size);
    }
    let result = self.cur_item;
    self.cur_item = self.cur_item.offset(1);
    result
  }
}

impl<T, const INIT_SIZE: usize> Drop for TNonFreePooledMemManager<T, INIT_SIZE> {
  #[inline(always)]
  fn drop(&mut self) {
    self.clear();
  }
}
