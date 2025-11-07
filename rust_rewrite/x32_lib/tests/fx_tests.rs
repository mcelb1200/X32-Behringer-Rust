
use x32_lib::fx;

#[test]
fn test_xfx_set_array() {
    // This is just a placeholder test to ensure the array is not accidentally truncated.
    println!("XFX1.len() = {}", fx::XFX1.len());
    assert_eq!(fx::XFX1.len(), 71);
}
