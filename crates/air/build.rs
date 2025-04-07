use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::Path;

const FIELD_MODULUS: u64 = 2147483647;
const FIXED_POINT_SCALE: f64 = 65536.0;

const TABLE_SIZE: usize = 360;

/// We want to precompute a sine lookup table for performance and convenience.
///
/// Computing `sin()` at runtime can be relatively expensive, especially if it's
/// called frequently in tight loops or in environments where floating-point
/// operations are limited.
///
/// Since the sine function is periodic and smooth, we can precompute `sin(i°)`
/// for `i` in `0..360` and store it in a flat `const` array. This lets us:
///
/// - Avoid computing sin at runtime
/// - Use fast lookup for `sin(angle_in_degrees)`
/// - Support wrapping automatically using modulo
///
/// For example:
///
/// ```text
/// Index:     0    1     2     3    ...   90    ...  359
/// Value:     0  0.017 0.035 0.052  ...   1.0   ... -0.017
/// ```
///
/// This table will be written as a Rust source file at build time (`OUT_DIR/sin_table.rs`)
/// and included in your crate via:
///
/// ```rust
/// include!(concat!(env!("OUT_DIR"), "/sin_table.rs"));
/// ```
///
/// Then, you can use:
///
/// ```rust
/// fn sin_lookup_deg(deg: usize) -> f32 {
///     SIN_TABLE[deg % 360]
/// }
/// ```
///
/// This strategy gives you full performance and avoids runtime trigonometric computation.
///
/// ---

fn float_to_fixed_u32(val: f64) -> u32 {
    let scaled = (val * FIXED_POINT_SCALE).round() as i64;

    // Handle negative values by mapping to field space
    let adjusted = if scaled < 0 {
        (FIELD_MODULUS as i64 + scaled) as u32
    } else {
        scaled as u32
    };

    adjusted
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("Missing OUT_DIR");
    let dest_path = Path::new(&out_dir).join("sin_table.rs");

    let mut table_entries = Vec::with_capacity(TABLE_SIZE);

    for i in 0..TABLE_SIZE {
        let degrees = i as f64;
        let sin_val = (degrees * PI / 180.0).sin();
        let input = i as u32;
        let output = float_to_fixed_u32(sin_val);

        table_entries.push(format!(
            "(BaseField::from_u32_unchecked({}), BaseField::from_u32_unchecked({}))",
            input, output
        ));
    }

    let table_str = format!(
        "/// Auto-generated sine table (angle degrees → sin value as field-point)\n\
         use stwo_prover::core::fields::m31::BaseField;\n\
         pub const SIN_LOOKUP_TABLE: &[(BaseField, BaseField)] = &[\n    {}\n];",
        table_entries.join(",\n    ")
    );

    fs::write(dest_path, table_str).expect("Failed to write sin_table.rs");
    println!("cargo:rerun-if-changed=build.rs");
}
