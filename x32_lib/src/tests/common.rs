#[cfg(test)]
mod tests {
    use crate::common::*;

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
