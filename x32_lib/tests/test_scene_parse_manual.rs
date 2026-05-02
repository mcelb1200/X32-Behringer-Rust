use x32_lib::scene_parse::parse_scene_line;
use osc_lib::OscArg;

#[test]
fn test_multiple_args() {
    let result = parse_scene_line("/ch/01/eq 1 20.0 1.0");
    println!("{:?}", result);
}
