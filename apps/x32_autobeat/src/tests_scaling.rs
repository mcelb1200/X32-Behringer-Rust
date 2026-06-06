use crate::scaling::{afine2float, log2float, ratio2float};

#[test]
fn test_log2float() {
    // Test HALL decay range: 0.2 to 5.0, log factor 3.218895825
    // log2float(0.2, 3.218...) should be close to 0.0
    let val_min = log2float(0.2, 0.2, 3.218895825);
    assert!((val_min - 0.0).abs() < 0.001);

    // log2float(5.0, 3.218...) should be close to 1.0
    let val_max = log2float(5.0, 0.2, 3.218895825);
    assert!((val_max - 1.0).abs() < 0.001);

    // Test mid value
    // ln(1.0 / 0.2) / 3.218... = 1.609 / 3.218 = 0.5
    let val_mid = log2float(1.0, 0.2, 3.218895825);
    assert!((val_mid - 0.5).abs() < 0.001);
}

#[test]
fn test_afine2float() {
    // Test ROOM size range: 4 to 76, range 72
    // afine2float(4, 72) should be 0.0
    let val_min = afine2float(4.0, 4.0, 72.0);
    assert!((val_min - 0.0).abs() < 0.001);

    // afine2float(76, 72) should be 1.0
    let val_max = afine2float(76.0, 4.0, 72.0);
    assert!((val_max - 1.0).abs() < 0.001);

    // afine2float(40, 72) -> (40-4)/72 = 36/72 = 0.5
    let val_mid = afine2float(40.0, 4.0, 72.0);
    assert!((val_mid - 0.5).abs() < 0.001);
}

#[test]
fn test_ratio2float() {
    // Test PreDelay 0-200
    let val_min = ratio2float(0.0, 200.0);
    assert!((val_min - 0.0).abs() < 0.001);

    let val_max = ratio2float(200.0, 200.0);
    assert!((val_max - 1.0).abs() < 0.001);

    let val_mid = ratio2float(100.0, 200.0);
    assert!((val_mid - 0.5).abs() < 0.001);
}
