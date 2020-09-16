// Original author: Akira1364
// Year: 2020
// License: MIT

// Note: This was written to simply be as similar as possible to my other-language versions of it
// in every way, with no specific attempt made to not use unsafe "just because this is Rust".

#![allow(incomplete_features, non_upper_case_globals)]
#![feature(alloc_layout_extra, const_fn, const_generics, const_panic)]

pub use crate::pooledmm::TNonFreePooledMemManager;
use rayon::prelude::*;

mod pooledmm;

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

type TNodePool = TNonFreePooledMemManager<TNode, 64>;

impl TNode {
  #[inline(always)]
  fn check_node(node: *mut TNode) -> i32 {
    // `node` is never itself null when passed into this function.
    let node_ref = unsafe { &mut *node };
    if !(node_ref.right.is_null() && node_ref.left.is_null()) {
      return 1 + TNode::check_node(node_ref.right) + TNode::check_node(node_ref.left);
    }
    1
  }

  // This can't be `&mut` instead of `*mut` due to the lifetime / borrowing rules.
  #[inline(always)]
  fn make_tree(depth: i32, node_pool: *mut TNodePool) -> *mut TNode {
    let res_ref = unsafe { &mut *((*node_pool).new_item()) };
    if depth > 0 {
      res_ref.right = TNode::make_tree(depth - 1, node_pool);
      res_ref.left = TNode::make_tree(depth - 1, node_pool);
    }
    res_ref as *mut T
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
  // How exactly making use of fixed-size static arrays is "unsafe" is beyond me, but...
  let slice = unsafe { &mut data[0..high_index as usize] };
  slice.par_iter_mut().enumerate().for_each(|(i, item)| {
    item.depth = min_depth + i as u8 * 2;
    item.iterations = 1 << (max_depth - i as u8 * 2);
    let mut inner_pool = TNodePool::new();
    for _ in 1..item.iterations + 1 {
      item.check += TNode::check_node(TNode::make_tree(item.depth as i32, &mut inner_pool));
      inner_pool.clear();
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
  pool.clear();
}
