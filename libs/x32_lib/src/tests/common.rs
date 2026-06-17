#[cfg(test)]
mod tests {
    use crate::common::*;

    #[test]
    fn test_command_format() {
        let int_format = CommandFormat::Int;
        let float_format = CommandFormat::Float;
        let string_format = CommandFormat::String;
        let list = &["A", "B"];
        let string_list_format = CommandFormat::StringList(list);

        let cloned_int = int_format.clone();
        assert!(matches!(cloned_int, CommandFormat::Int));

        let cloned_float = float_format.clone();
        assert!(matches!(cloned_float, CommandFormat::Float));

        let cloned_string = string_format.clone();
        assert!(matches!(cloned_string, CommandFormat::String));

        let cloned_string_list = string_list_format.clone();
        if let CommandFormat::StringList(cloned_list) = cloned_string_list {
            assert_eq!(cloned_list, list);
        } else {
            panic!("Expected CommandFormat::StringList");
        }
    }

    #[test]
    fn test_command_value() {
        let int_val = CommandValue::Int(42);
        let float_val = CommandValue::Float(3.14);
        let str_val = CommandValue::String("hello".to_string());
        let none_val = CommandValue::None;

        if let CommandValue::Int(val) = int_val.clone() {
            assert_eq!(val, 42);
        } else {
            panic!("Expected CommandValue::Int");
        }

        if let CommandValue::Float(val) = float_val.clone() {
            assert_eq!(val, 3.14);
        } else {
            panic!("Expected CommandValue::Float");
        }

        if let CommandValue::String(val) = str_val.clone() {
            assert_eq!(val, "hello");
        } else {
            panic!("Expected CommandValue::String");
        }

        assert!(matches!(none_val.clone(), CommandValue::None));
    }

    #[test]
    fn test_command_flags() {
        let flag_get = CommandFlags::F_GET;
        let flag_set = CommandFlags::F_SET;
        let flag_xet = CommandFlags::F_XET;
        let flag_npr = CommandFlags::F_NPR;
        let flag_fnd = CommandFlags::F_FND;

        assert_eq!(flag_get.bits(), 0x0001);
        assert_eq!(flag_set.bits(), 0x0002);
        assert_eq!(flag_xet.bits(), 0x0003);
        assert_eq!(flag_npr.bits(), 0x0004);
        assert_eq!(flag_fnd.bits(), 0x0008);

        // F_XET should contain F_GET and F_SET
        assert!(flag_xet.contains(CommandFlags::F_GET));
        assert!(flag_xet.contains(CommandFlags::F_SET));
        assert_eq!(flag_xet, flag_get | flag_set);

        // Derives test
        assert_eq!(flag_get.clone(), CommandFlags::F_GET);
        assert!(flag_get != flag_set);
    }

    #[test]
    fn test_on_enum() {
        assert_eq!(On::from_id(0), Some(On::Off));
        assert_eq!(On::from_id(1), Some(On::On));
        assert_eq!(On::from_id(2), None);
    }

    #[test]
    fn test_color_enum() {
        assert_eq!(Color::from_id(0), Some(Color::Off));
        assert_eq!(Color::from_id(1), Some(Color::Red));
        assert_eq!(Color::from_id(15), Some(Color::WhiteInverted));
        assert_eq!(Color::from_id(16), None);
    }

    #[test]
    fn test_eq_type_enum() {
        assert_eq!(EqType::from_id(0), Some(EqType::Lcut));
        assert_eq!(EqType::from_id(1), Some(EqType::LShv));
        assert_eq!(EqType::from_id(2), Some(EqType::Peq));
        assert_eq!(EqType::from_id(3), Some(EqType::Veq));
        assert_eq!(EqType::from_id(4), Some(EqType::HShv));
        assert_eq!(EqType::from_id(5), Some(EqType::Hcut));
        assert_eq!(EqType::from_id(6), None);
        assert_eq!(EqType::from_id(255), None);
    }

    #[test]
    fn test_insert_position_enum() {
        assert_eq!(InsertPosition::from_id(0), Some(InsertPosition::Pre));
        assert_eq!(InsertPosition::from_id(1), Some(InsertPosition::Post));
        assert_eq!(InsertPosition::from_id(2), None);
    }

    #[test]
    fn test_insert_selection_enum() {
        assert_eq!(InsertSelection::from_id(0), Some(InsertSelection::Off));
        assert_eq!(InsertSelection::from_id(1), Some(InsertSelection::Fx1L));
        assert_eq!(InsertSelection::from_id(2), Some(InsertSelection::Fx1R));
        assert_eq!(InsertSelection::from_id(3), Some(InsertSelection::Fx2L));
        assert_eq!(InsertSelection::from_id(4), Some(InsertSelection::Fx2R));
        assert_eq!(InsertSelection::from_id(5), Some(InsertSelection::Fx3L));
        assert_eq!(InsertSelection::from_id(6), Some(InsertSelection::Fx3R));
        assert_eq!(InsertSelection::from_id(7), Some(InsertSelection::Fx4L));
        assert_eq!(InsertSelection::from_id(8), Some(InsertSelection::Fx4R));
        assert_eq!(InsertSelection::from_id(9), Some(InsertSelection::Fx5L));
        assert_eq!(InsertSelection::from_id(10), Some(InsertSelection::Fx5R));
        assert_eq!(InsertSelection::from_id(11), Some(InsertSelection::Fx6L));
        assert_eq!(InsertSelection::from_id(12), Some(InsertSelection::Fx6R));
        assert_eq!(InsertSelection::from_id(13), Some(InsertSelection::Fx7L));
        assert_eq!(InsertSelection::from_id(14), Some(InsertSelection::Fx7R));
        assert_eq!(InsertSelection::from_id(15), Some(InsertSelection::Fx8L));
        assert_eq!(InsertSelection::from_id(16), Some(InsertSelection::Fx8R));
        assert_eq!(InsertSelection::from_id(17), Some(InsertSelection::Aux1));
        assert_eq!(InsertSelection::from_id(18), Some(InsertSelection::Aux2));
        assert_eq!(InsertSelection::from_id(19), Some(InsertSelection::Aux3));
        assert_eq!(InsertSelection::from_id(20), Some(InsertSelection::Aux4));
        assert_eq!(InsertSelection::from_id(21), Some(InsertSelection::Aux5));
        assert_eq!(InsertSelection::from_id(22), Some(InsertSelection::Aux6));
        assert_eq!(InsertSelection::from_id(23), None);
        assert_eq!(InsertSelection::from_id(255), None);
    }

    #[test]
    fn test_fx_source_enum() {
        assert_eq!(FxSource::Off.to_id(), 0);
        assert_eq!(FxSource::MixBus(1).to_id(), 1);
        assert_eq!(FxSource::Bus(1).to_id(), 17);
        assert_eq!(FxSource::Mtx(1).to_id(), 33);
        assert_eq!(FxSource::Main(1).to_id(), 39);
        assert_eq!(FxSource::Group(1).to_id(), 41);

        assert_eq!(FxSource::from_id(0), Some(FxSource::Off));
        assert_eq!(FxSource::from_id(1), Some(FxSource::MixBus(1)));
        assert_eq!(FxSource::from_id(17), Some(FxSource::Bus(1)));
        assert_eq!(FxSource::from_id(33), Some(FxSource::Mtx(1)));
        assert_eq!(FxSource::from_id(39), Some(FxSource::Main(1)));
        assert_eq!(FxSource::from_id(41), Some(FxSource::Group(1)));
        assert_eq!(FxSource::from_id(50), None);
    }
}
