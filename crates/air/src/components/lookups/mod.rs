use numerair::Fixed;
use serde::{Deserialize, Serialize};
use sin::{table::SinLookup, SinLookupElements};
use stwo_prover::core::channel::Channel;

use crate::utils::calculate_log_size;

pub mod sin;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Range(pub Fixed, pub Fixed);

/// Counts the exact number of **distinct, non‑zero** integer inputs that
/// will populate this lookup column **before** the column is
/// padded to the next power‑of‑two.
fn value_count(ranges: &Vec<Range>) -> u32 {
    ranges.iter().map(|r| (r.1 .0 - r.0 .0 + 1) as u32).sum()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Lookups {
    pub sin: Option<SinLookup>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Layout {
    pub ranges: Vec<Range>,
    pub log_size: u32,
}

impl Layout {
    pub fn new(ranges: Vec<Range>) -> Self {
        let log_size = calculate_log_size(value_count(&ranges) as usize);
        Self { ranges, log_size }
    }

    /// Finds the index of a value in the LUT.
    pub fn find_index(&self, target: i64) -> Option<usize> {
        // Binary search to find the range containing the target
        match self.find_containing_range(target) {
            Some((range_idx, range)) => {
                // Calculate the cumulative count of values before this range
                let mut cumulative_count = 0;
                for i in 0..range_idx {
                    let r = &self.ranges[i];
                    cumulative_count += (r.1 .0 - r.0 .0 + 1) as usize;
                }

                // Add the offset within the found range
                let offset = (target - range.0 .0) as usize;
                Some(cumulative_count + offset)
            }
            None => None,
        }
    }

    /// Find which range contains the target value.
    fn find_containing_range(&self, target: i64) -> Option<(usize, &Range)> {
        // Early check for empty ranges
        if self.ranges.is_empty() {
            return None;
        }

        // Binary search to find the correct range
        let mut left = 0;
        let mut right = self.ranges.len() - 1;

        while left <= right {
            let mid = left + (right - left) / 2;
            let range = &self.ranges[mid];

            // Check if target is in this range
            if target >= range.0 .0 && target <= range.1 .0 {
                return Some((mid, range));
            }

            // Adjust search boundaries
            if target < range.0 .0 {
                // Target is before this range
                if mid == 0 {
                    break; // Can't go left further
                }
                right = mid - 1;
            } else {
                // Target is after this range
                if mid == self.ranges.len() - 1 {
                    break; // Can't go right further
                }
                left = mid + 1;
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
pub struct LookupElements {
    pub sin: SinLookupElements,
}

impl LookupElements {
    pub fn draw(channel: &mut impl Channel) -> Self {
        Self {
            sin: SinLookupElements::draw(channel),
        }
    }
}
