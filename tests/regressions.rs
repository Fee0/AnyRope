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

#[test]
fn append_keeps_ropes_with_zero_measure() {
    let mut rope = Rope::<Width, 4, 4>::from_slice(&widths(&[1]));
    rope.append(Rope::from_slice(&widths(&[0, 0])));
    assert_eq!(contents(&rope), widths(&[1, 0, 0]));

    let mut rope = Rope::<Width, 4, 4>::new();
    rope.append(Rope::from_slice(&widths(&[0, 0])));
    assert_eq!(contents(&rope), widths(&[0, 0]));

    let mut rope = Rope::<Width, 4, 4>::from_slice(&widths(&[0, 0, 0]));
    rope.append(Rope::from_slice(&widths(&[0; 20])));
    rope.assert_integrity();
    rope.assert_invariants();
    assert_eq!(contents(&rope), widths(&[0; 23]));

    let mut rope = Rope::<Width, 4, 4>::from_slice(&widths(&[0; 20]));
    rope.append(Rope::new());
    assert_eq!(contents(&rope), widths(&[0; 20]));
}

#[test]
fn large_insert_slice_keeps_zero_measure_elements() {
    // Slices longer than `LEAF_CAP * 6` are spliced in through `append`.
    let mut rope = Rope::<Width, 4, 4>::from_slice(&widths(&[1]));
    rope.insert_slice(1, &widths(&[0; 30]), usize::cmp);
    rope.assert_integrity();
    rope.assert_invariants();
    assert_eq!(rope.len(), 31);
}

#[test]
fn leaves_respect_leaf_cap_when_it_differs_from_branch_cap() {
    let mut builder = any_rope::RopeBuilder::<Width, 4, 32>::new();
    for width in widths(&[1, 2, 3, 4, 5]) {
        builder.append(width);
    }
    let rope = builder.finish();
    rope.assert_integrity();
    rope.assert_invariants();
    assert!(rope.chunks().all(|chunk| chunk.len() <= 4));
    assert_eq!(contents(&rope), widths(&[1, 2, 3, 4, 5]));

    // Undersized leaves must still merge when `LEAF_CAP > BRANCH_CAP`.
    let mut rope = Rope::<Width, 8, 4>::from_slice(&widths(&[1; 40]));
    rope.remove_inclusive(3..37, usize::cmp);
    rope.assert_integrity();
    rope.assert_invariants();
    assert!(rope.chunks().all(|chunk| chunk.len() <= 8));
    assert_eq!(contents(&rope), widths(&[1; 6]));
}

#[test]
fn remove_exclusive_inside_one_element_does_nothing() {
    let mut rope = Rope::<Width>::from_slice(&widths(&[4]));
    rope.remove_exclusive(1..2, usize::cmp);
    assert_eq!(contents(&rope), widths(&[4]));

    // The array from the `remove_exclusive` docs; 4..5 lies inside the `Width(3)`.
    let mut rope = Rope::<Width>::from_slice(&widths(&[1, 2, 3, 0, 0, 2, 1]));
    assert!(rope.try_remove_exclusive(4..5, usize::cmp).is_ok());
    assert_eq!(contents(&rope), widths(&[1, 2, 3, 0, 0, 2, 1]));

    // The same, deep inside a multi-level tree.
    let mut model = widths(&[1; 30]);
    model[17] = Width(10);
    let mut rope = Rope::<Width, 4, 4>::from_slice(&model);
    rope.remove_exclusive(19..24, usize::cmp);
    rope.assert_integrity();
    rope.assert_invariants();
    assert_eq!(contents(&rope), model);
}

#[test]
fn ord_agrees_with_eq_across_chunkings() {
    let model = widths(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

    // Same contents, chunked differently.
    let mut chunked = Rope::<Width, 4, 4>::new();
    for chunk in model.chunks(3) {
        chunked.append(Rope::from_slice(chunk));
    }
    let built = Rope::<Width, 4, 4>::from_slice(&model);
    assert_ne!(
        chunked.chunks().map(<[Width]>::len).collect::<Vec<_>>(),
        built.chunks().map(<[Width]>::len).collect::<Vec<_>>(),
        "test needs ropes with different chunking"
    );

    assert_eq!(chunked, built);
    assert_eq!(chunked.cmp(&built), std::cmp::Ordering::Equal);
    assert_eq!(built.cmp(&chunked), std::cmp::Ordering::Equal);

    let mut greater = model.clone();
    *greater.last_mut().unwrap() = Width(13);
    let greater = Rope::<Width, 4, 4>::from_slice(&greater);
    assert_eq!(chunked.cmp(&greater), std::cmp::Ordering::Less);
    assert_eq!(greater.cmp(&chunked), std::cmp::Ordering::Greater);
}

#[test]
fn nested_measure_slice_is_relative_to_the_outer_slice() {
    let model: Vec<Width> = (0..64).map(|i| Width([1, 2, 4, 1, 3, 5][i % 6])).collect();
    let rope = Rope::<Width, 4, 4>::from_slice(&model);
    let total = rope.measure();

    // Outer slices starting on element boundaries, so that their measure 0 is
    // unambiguous.
    let starts = model.iter().scan(0, |offset, width| {
        let start = *offset;
        *offset += width.0;
        Some(start)
    });
    for outer_start in starts.step_by(5) {
        let outer = rope.measure_slice(outer_start..total, usize::cmp);
        let outer_measure = outer.measure();
        for (start, end) in [(0, 1), (3, 9), (5, 20), (0, outer_measure), (2, outer_measure)] {
            if end > outer_measure {
                continue;
            }
            let nested = outer.measure_slice(start..end, usize::cmp);
            let direct = rope.measure_slice(outer_start + start..outer_start + end, usize::cmp);
            assert_eq!(nested, direct, "outer {outer_start}.., nested {start}..{end}");
            assert_eq!(
                outer.get_measure_slice(start..end, usize::cmp),
                Some(direct),
                "outer {outer_start}.., nested {start}..{end}"
            );
        }
    }
}
