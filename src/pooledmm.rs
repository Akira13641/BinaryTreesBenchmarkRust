// Original author: Akira1364
// Year: 2020
// License: MIT

use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
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
  pub fn new_item(&mut self) -> &mut T {
    if self.cur_item == self.end_item {
      self.cur_size += self.cur_size;
      // The next bit will fail with `attempt to divide by zero` if `T` is a ZST, which seems
      // appropriate enough considering the call to `alloc_zeroed` immediately afterwards would also
      // fail if actually reached with a ZST.
      let layout = Layout::new::<T>().repeat_packed(self.cur_size).unwrap();
      self.cur_item = unsafe { alloc_zeroed(layout) as *mut T };
      // Generally I feel like if `cur_item` is actually null the user probably has bigger issues to
      // deal with, but properly checking for it doesn't make things noticeably slower so there's no
      // real reason not to.
      if self.cur_item.is_null() {
        handle_alloc_error(layout)
      } else {
        self.items.push((self.cur_item, layout));
      }
      self.end_item = self.cur_item;
      self.end_item = unsafe { self.end_item.add(self.cur_size) };
    }
    let result = self.cur_item;
    unsafe {
      self.cur_item = self.cur_item.offset(1);
      &mut *result
    }
  }

  // Note that this enumerates *all allocated* items, i.e. a number which is always greater than
  // both `items.len()` and the number of times that `new_item()` has been called.
  #[inline]
  pub fn enumerate_items<F>(&mut self, mut fun: F)
  where F: FnMut(&mut T) -> () {
    let length = self.items.len();
    if length > 0 {
      let mut size = INIT_SIZE;
      for i in 0..length {
        size += size;
        unsafe {
          let mut p = self.items.get_unchecked_mut(i).0;
          let mut last = p;
          last = last.add(size);
          if i == length - 1 {
            last = self.end_item;
          }
          while p != last {
            fun(&mut *p);
            p = p.offset(1);
          }
        }
      }
    }
  }
}

impl<T, const INIT_SIZE: usize> Drop for TNonFreePooledMemManager<T, INIT_SIZE> {
  #[inline(always)]
  fn drop(&mut self) {
    self.clear();
  }
}
