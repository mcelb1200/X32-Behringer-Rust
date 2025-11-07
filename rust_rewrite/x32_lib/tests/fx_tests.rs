use x32_lib::fx;

#[test]
fn test_fx_data_port() {
    // Verify that the XFX1 array has the correct number of elements.
    assert_eq!(fx::XFX1.len(), 7);
}
