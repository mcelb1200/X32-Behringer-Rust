
/// Converts a linear value to a float [0.0, 1.0] using logarithmic scaling.
/// Based on C macro `log2float`.
///
/// Formula: `y = ln(x / min) / range_log`
/// where `range_log = ln(max / min)`
pub fn log2float(val: f32, min: f32, range_log: f32) -> f32 {
    if val <= 0.0 { return 0.0; }
    let res = (val / min).ln() / range_log;
    res.clamp(0.0, 1.0)
}

/// Converts a linear value to a float [0.0, 1.0] using affine (linear) scaling.
/// Based on C macro `afine2float`.
///
/// Formula: `y = (x - min) / range`
/// where `range = max - min`
pub fn afine2float(val: f32, min: f32, range: f32) -> f32 {
    let res = (val - min) / range;
    res.clamp(0.0, 1.0)
}

/// Converts a value to a float [0.0, 1.0] using simple ratio scaling.
/// Based on C macro `ratio2float`.
///
/// Formula: `y = x / max`
/// Assumes min is 0.
pub fn ratio2float(val: f32, max: f32) -> f32 {
    let res = val / max;
    res.clamp(0.0, 1.0)
}
