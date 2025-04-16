use std::collections::HashMap;

use luminair_air::preprocessed::Range;

pub struct CircuitSettings {
    pub(crate) lut_ranges: HashMap<usize /* node_id */, Range>,
}
