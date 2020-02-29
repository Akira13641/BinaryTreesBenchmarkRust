// Original author: Akira1364
// Year: 2020
// License: MIT

// Note: This was written to simply be as similar as possible to my other-language versions of it
// in every way, with no specific attempt made to not use unsafe "just because this is Rust".

#![allow(incomplete_features)]
#![allow(non_upper_case_globals)]
#![feature(alloc_layout_extra)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(core_intrinsics)]

pub use crate::pooledmm::TNonFreePooledMemManager;
mod pooledmm;

use rayon::prelude::*;

#[derive(Copy, Clone)]
struct TDataRec {
  depth: u8,
  iterations: i32,
  check: i32,
}

struct TNode {
  left: *mut TNode,
  right: *mut TNode,
}

type TNodePool = TNonFreePooledMemManager<TNode, 32>;

impl TNode {
  #[inline(always)]
  fn check_node(node: *mut TNode) -> i32 {
    unsafe {
      // `node` is never itself null when passed into this function.
      if !((*node).right.is_null() && (*node).left.is_null()) {
        return 1 + TNode::check_node((*node).right) + TNode::check_node((*node).left);
      }
    }
    return 1;
  }

  // This can't be `&mut` instead of `*mut` due to the lifetime / borrowing rules.
  #[inline(always)]
  fn make_tree(depth: i32, mp: *mut TNodePool) -> *mut TNode {
    let result = unsafe { (*mp).new_item() };
    if depth > 0 {
      result.right = TNode::make_tree(depth - 1, mp);
      result.left = TNode::make_tree(depth - 1, mp);
    }
    result
  }
}

static min_depth: u8 = 4;

static mut data: [TDataRec; 9] = [TDataRec {
  depth: 0,
  iterations: 0,
  check: 0,
}; 9];

fn main() {
  let n = std::env::args()
    .nth(1)
    .and_then(|n| n.parse().ok())
    .unwrap_or(10);

  let max_depth = if min_depth + 2 > n { min_depth + 2 } else { n };

  // Create and destroy a tree of depth `max_depth + 1`.
  let mut pool = TNodePool::new();
  println!(
    "stretch tree of depth {}\t check: {}",
    max_depth + 1,
    TNode::check_node(TNode::make_tree(max_depth as i32 + 1, &mut pool))
  );
  pool.clear();

  // Create a "long lived" tree of depth `max_depth`.
  let tree = TNode::make_tree(max_depth as i32, &mut pool);

  // While the tree stays live, create multiple trees. Local data is stored in
  // the `data` variable.
  let high_index = (max_depth - min_depth) / 2 + 1;
  let slice = unsafe { &mut data[0..high_index as usize] };
  slice.par_iter_mut().enumerate().for_each(|(i, item)| {
    item.depth = min_depth + i as u8 * 2;
    item.iterations = 1 << max_depth - i as u8 * 2;
    let mut ipool = TNodePool::new();
    for _ in 1..item.iterations + 1 {
      item.check += TNode::check_node(TNode::make_tree(item.depth as i32, &mut ipool));
      ipool.clear();
    }
  });

  // Display the results.
  for item in slice {
    println!(
      "{}\t trees of depth {}\t check: {}",
      item.iterations, item.depth, item.check
    );
  }

  // Check and destroy the long lived tree.
  println!(
    "long lived tree of depth {}\t check: {}",
    max_depth,
    TNode::check_node(tree)
  );

  // `pool` is cleared on drop right around here.
}
