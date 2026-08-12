#![allow(missing_docs)]
use anyhow::Result;
use x32_testparams::{ParamDefinition, ParamType, ParamValue, parse_file_content, parse_line};

#[test]
fn test_parse_char() -> Result<()> {
    let mut defs = vec![ParamDefinition::new("my_char", ParamType::Char)];

    parse_line("my_char=A", &mut defs)?;
    assert_eq!(defs[0].value, Some(ParamValue::Char('A')));

    Ok(())
}

#[test]
fn test_parse_int() -> Result<()> {
    let mut defs = vec![ParamDefinition::new("my_int", ParamType::Int)];

    parse_line("my_int=42", &mut defs)?;
    assert_eq!(defs[0].value, Some(ParamValue::Int(42)));

    Ok(())
}

#[test]
fn test_parse_float() -> Result<()> {
    let mut defs = vec![ParamDefinition::new("my_float", ParamType::Float)];

    parse_line("my_float=3.14", &mut defs)?;
    assert_eq!(defs[0].value, Some(ParamValue::Float(3.14)));

    Ok(())
}

#[test]
fn test_parse_string() -> Result<()> {
    let mut defs = vec![ParamDefinition::new("my_str", ParamType::String)];

    parse_line("my_str=hello world", &mut defs)?;
    assert_eq!(defs[0].value, Some(ParamValue::String("hello world".to_string())));

    Ok(())
}

#[test]
fn test_parse_hex_string() -> Result<()> {
    let mut defs = vec![ParamDefinition::new("my_hstr", ParamType::HexString)];

    // Test params expects hex strings to be read byte by byte, separated by space maybe?
    // "f0 f1 f2" -> vec![0xf0, 0xf1, 0xf2]
    parse_line("my_hstr=f0 f1 f2", &mut defs)?;
    assert_eq!(defs[0].value, Some(ParamValue::HexString(vec![0xf0, 0xf1, 0xf2])));

    Ok(())
}

#[test]
fn test_parse_yesno() -> Result<()> {
    let mut defs = vec![
        ParamDefinition::new("is_yes1", ParamType::YesNo),
        ParamDefinition::new("is_yes2", ParamType::YesNo),
        ParamDefinition::new("is_no1", ParamType::YesNo),
        ParamDefinition::new("is_no2", ParamType::YesNo),
    ];

    parse_line("is_yes1=Y", &mut defs)?;
    parse_line("is_yes2=yes", &mut defs)?;
    parse_line("is_no1=N", &mut defs)?;
    parse_line("is_no2=no", &mut defs)?;

    assert_eq!(defs[0].value, Some(ParamValue::YesNo(true)));
    assert_eq!(defs[1].value, Some(ParamValue::YesNo(true)));
    assert_eq!(defs[2].value, Some(ParamValue::YesNo(false)));
    assert_eq!(defs[3].value, Some(ParamValue::YesNo(false)));

    Ok(())
}

#[test]
fn test_parse_file_content() -> Result<()> {
    let content = "\
# This is a comment
my_int=100
my_str=test string
# Another comment

my_float=1.23
";

    let mut defs = vec![
        ParamDefinition::new("my_int", ParamType::Int),
        ParamDefinition::new("my_str", ParamType::String),
        ParamDefinition::new("my_float", ParamType::Float),
    ];

    parse_file_content(content, &mut defs)?;

    assert_eq!(defs[0].value, Some(ParamValue::Int(100)));
    assert_eq!(defs[1].value, Some(ParamValue::String("test string".to_string())));
    assert_eq!(defs[2].value, Some(ParamValue::Float(1.23)));

    Ok(())
}

#[test]
fn test_line_parse_unknown_param_returns_error() {
    let mut defs = vec![ParamDefinition::new("known", ParamType::Int)];
    let res = parse_line("unknown=10", &mut defs);
    assert!(res.is_err());
}
