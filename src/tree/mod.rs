mod branch_children;
mod leaf_slice;
mod node;
mod slice_info;

pub(crate) use self::{
    branch_children::BranchChildren, leaf_slice::LeafSlice, node::Node, slice_info::SliceInfo,
};

// Type used for storing tree metadata, such as indices and widths.
pub(crate) type Count = u64;

/// Default number of elements stored inline in each leaf node.
pub const DEFAULT_LEAF_CAP: usize = 96;

/// Default number of child pointers stored inline in each branch node.
pub const DEFAULT_BRANCH_CAP: usize = 32;

pub(crate) const fn min_len(leaf_cap: usize) -> usize {
    (leaf_cap / 2) - (leaf_cap / 32)
}

pub(crate) const fn min_children(branch_cap: usize) -> usize {
    branch_cap / 2
}

pub(crate) fn assert_valid_capacities<const LEAF_CAP: usize, const BRANCH_CAP: usize>() {
    assert!(
        LEAF_CAP >= 4,
        "LEAF_CAP must be at least 4 to preserve rope balancing invariants"
    );
    assert!(
        BRANCH_CAP >= 4,
        "BRANCH_CAP must be at least 4 to preserve rope balancing invariants"
    );
}
