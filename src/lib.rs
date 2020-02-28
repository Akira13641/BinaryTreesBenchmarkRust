// Original author: Akira1364
// Year: 2020
// License: MIT

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
      if !((*node).right.is_null() && (*node).left.is_null()) {
        return 1 + TNode::check_node((*node).right) + TNode::check_node((*node).left);
      }
    }
    return 1;
  }

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
    .unwrap_or(21);
    
  let max_depth = if min_depth + 2 > n { min_depth + 2 } else { n };

  let mut pool = TNodePool::new();
  
  println!(
    "stretch tree of depth {}\t check: {}",
    max_depth + 1,
    TNode::check_node(TNode::make_tree(max_depth as i32 + 1, &mut pool))
  );
  
  pool.clear();

  let tree = TNode::make_tree(max_depth as i32, &mut pool);

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
  
  for item in slice {
    println!(
      "{}\t trees of depth {}\t check: {}",
      item.iterations, item.depth, item.check
    );
  }

  println!(
    "long lived tree of depth {}\t check: {}",
    max_depth,
    TNode::check_node(tree)
  );
}

