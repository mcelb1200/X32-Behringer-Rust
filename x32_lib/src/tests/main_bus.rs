#[cfg(test)]
mod tests {
    use crate::main_bus::*;
    use osc_lib::OscArg;

    #[test]
    fn test_set_st_name() {
        let (address, args) = set_st_name("MAIN ST");
        assert_eq!(address, "/main/st/config/name");
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], OscArg::String("MAIN ST".to_string()));
    }
}
