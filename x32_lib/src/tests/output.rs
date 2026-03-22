#[cfg(test)]
mod tests {
    use crate::command::output::*;
    use osc_lib::OscArg;

    #[test]
    fn test_main_output_source() {
        assert_eq!(main_output_source(1), "/outputs/main/01/src");
        assert_eq!(main_output_source(16), "/outputs/main/16/src");
    }

    #[test]
    fn test_set_main_output_source() {
        let (address, args) = set_main_output_source(1, 2);
        assert_eq!(address, "/outputs/main/01/src");
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], OscArg::Int(2));

        let (address, args) = set_main_output_source(16, 42);
        assert_eq!(address, "/outputs/main/16/src");
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], OscArg::Int(42));
    }
}
