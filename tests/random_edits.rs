//! Random edits checked against a plain `Vec` after every step.

use any_rope::{Rope, Width};
use rand::{Rng, SeedableRng, rngs::StdRng};

const SEEDS: u64 = 2_000;
const STEPS: usize = 60;
const MAX_WIDTH: usize = 9;

/// Measure at which element `index` starts.
fn offset_of(model: &[Width], index: usize) -> usize {
    model[..index].iter().map(|width| width.0).sum()
}

/// Indices of the elements overlapping `start..end`.
fn overlapping(model: &[Width], start: usize, end: usize) -> std::ops::Range<usize> {
    let mut offset = 0;
    let mut first = None;
    let mut last = 0;
    for (index, width) in model.iter().enumerate() {
        if offset < end && offset + width.0 > start {
            first.get_or_insert(index);
            last = index + 1;
        }
        offset += width.0;
    }
    first.map_or(0..0, |first| first..last)
}

fn random_edits<const LEAF_CAP: usize, const BRANCH_CAP: usize>(seed: u64) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut rope = Rope::<Width, LEAF_CAP, BRANCH_CAP>::new();
    let mut model: Vec<Width> = Vec::new();
    let mut snapshots = Vec::new();

    for step in 0..STEPS {
        let width = Width(rng.gen_range(1..=MAX_WIDTH));
        match rng.gen_range(0..7) {
            0 => {
                let index = rng.gen_range(0..=model.len());
                rope.insert(offset_of(&model, index), width, usize::cmp);
                model.insert(index, width);
            }
            1 => {
                rope.append(Rope::from(vec![width]));
                model.push(width);
            }
            2 => snapshots.push((rope.clone(), model.clone())),
            3 if !snapshots.is_empty() => {
                let index = rng.gen_range(0..snapshots.len());
                (rope, model) = snapshots.swap_remove(index);
            }
            _ if model.is_empty() => continue,
            4 => {
                let start = rng.gen_range(0..model.len());
                let end = rng.gen_range(start + 1..=model.len());
                let range = offset_of(&model, start)..offset_of(&model, end);
                rope.remove_exclusive(range, usize::cmp);
                model.drain(start..end);
            }
            _ => {
                let total = offset_of(&model, model.len());
                let start = rng.gen_range(0..total);
                let end = rng.gen_range(start + 1..=total);
                rope.remove_inclusive(start..end, usize::cmp);
                model.drain(overlapping(&model, start, end));
            }
        }

        let found: Vec<Width> = rope.iter().map(|(_, width)| width).collect();
        assert_eq!(
            found, model,
            "seed {seed}, step {step}, caps {LEAF_CAP}/{BRANCH_CAP}"
        );
        assert_eq!(rope.measure(), offset_of(&model, model.len()));
    }
}

#[test]
fn removing_everything_exclusively_leaves_an_editable_rope() {
    let mut rope = Rope::<Width, 4, 4>::from_slice(&[Width(1); 10]);
    rope.remove_exclusive(0..10, usize::cmp);
    rope.insert(0, Width(4), usize::cmp);

    assert_eq!(rope, [Width(4)].as_slice());
}

#[test]
fn inclusive_removal_starting_inside_an_element_fixes_the_right_seam() {
    let mut rope = Rope::<Width, 4, 4>::new();
    rope.insert(0, Width(3), usize::cmp);
    rope.append(Rope::from(vec![Width(6)]));
    rope.append(Rope::from(vec![Width(9)]));

    // Starts inside the 6, so it and everything after go, leaving less than `start`.
    rope.remove_inclusive(6..15, usize::cmp);

    assert_eq!(rope, [Width(3)].as_slice());
}

#[test]
#[cfg_attr(miri, ignore)]
fn random_edits_match_a_vec() {
    for seed in 0..SEEDS {
        random_edits::<4, 4>(seed);
        random_edits::<9, 5>(seed);
        random_edits::<96, 32>(seed);
    }
}
