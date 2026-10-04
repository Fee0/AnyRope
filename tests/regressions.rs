//! Regression tests for bugs found during review.

use std::{sync::mpsc, thread, time::Duration};

use any_rope::{Rope, Width};

fn widths(widths: &[usize]) -> Vec<Width> {
    widths.iter().copied().map(Width).collect()
}

fn contents<const LEAF_CAP: usize, const BRANCH_CAP: usize>(
    rope: &Rope<Width, LEAF_CAP, BRANCH_CAP>,
) -> Vec<Width> {
    rope.iter().map(|(_, width)| width).collect()
}

/// Runs `f` on another thread, failing if it doesn't finish in time instead of
/// hanging the test suite.
fn with_timeout<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || sender.send(f()).unwrap());
    receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("operation did not finish in time")
}

#[test]
fn insert_slice_terminates_with_minimum_leaf_cap() {
    let inserted = with_timeout(|| {
        let mut rope = Rope::<Width, 4, 4>::from_slice(&widths(&[1, 2]));
        rope.insert_slice(1, &widths(&[3, 4, 5, 6, 7, 8, 9]), usize::cmp);
        rope.assert_integrity();
        rope.assert_invariants();
        contents(&rope)
    });

    assert_eq!(inserted, widths(&[1, 3, 4, 5, 6, 7, 8, 9, 2]));
}
