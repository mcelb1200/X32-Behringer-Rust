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
        assert_eq!(EqType::from_id(5), Some(EqType::Hcut));
        assert_eq!(EqType::from_id(6), None);
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
        assert_eq!(InsertSelection::from_id(22), Some(InsertSelection::Aux6));
        assert_eq!(InsertSelection::from_id(23), None);
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
