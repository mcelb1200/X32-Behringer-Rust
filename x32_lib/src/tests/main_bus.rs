#[cfg(test)]
mod tests {
    use crate::common::Color;
    use crate::main_bus::*;
    use osc_lib::OscArg;

    #[test]
    fn test_set_st_name() {
        let (address, args) = set_st_name("MAIN ST");
        assert_eq!(address, "/main/st/config/name");
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], OscArg::String("MAIN ST".to_string()));
        let name = "Main LR";
        let (address, args) = set_st_name(name);

        assert_eq!(address, "/main/st/config/name");
        assert_eq!(args.len(), 1);

        match &args[0] {
            OscArg::String(s) => assert_eq!(s, name),
            _ => panic!("Expected OscArg::String"),
        }
    }

    #[test]
    fn test_set_st_color() {
        let (address, args) = set_st_color(Color::Red);

        assert_eq!(address, "/main/st/config/color");
        assert_eq!(args.len(), 1);

        match &args[0] {
            OscArg::Int(val) => assert_eq!(*val, 1),
            _ => panic!("Expected OscArg::Int"),
        }

        let (address, args) = set_st_color(Color::Off);

        assert_eq!(address, "/main/st/config/color");
        assert_eq!(args.len(), 1);

        match &args[0] {
            OscArg::Int(val) => assert_eq!(*val, 0),
            _ => panic!("Expected OscArg::Int"),
        }
    }
}
