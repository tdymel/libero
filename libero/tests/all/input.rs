use libero::{components::*, theme::*};

fn converts<T: Default + PartialEq + std::fmt::Debug + 'static>()
where
    Input<T>: From<T> + From<Option<T>>,
{
    assert_eq!(Input::from(T::default()), Input::Value(T::default()));
    assert_eq!(Input::<T>::from(None::<T>), Input::None);
}

/// 982: every `input_from_str!` enum also takes its own value, so
/// `direction: FlexDirection::Row` compiles like `direction: "row"`.
#[test]
fn every_str_enum_input_takes_its_own_value() {
    converts::<AnchorUnderline>();
    converts::<CalendarVariant>();
    converts::<CarouselAlign>();
    converts::<CascaderLayout>();
    converts::<ChoiceVariant>();
    converts::<ColorFormat>();
    converts::<DrawerAnchor>();
    converts::<FileFieldVariant>();
    converts::<FlexDirection>();
    converts::<FlexWrap>();
    converts::<GridSpan>();
    converts::<HeaderPosition>();
    converts::<ImageFit>();
    converts::<ImageListVariant>();
    converts::<ImageLoading>();
    converts::<LabelPosition>();
    converts::<LoaderVariant>();
    converts::<Orientation>();
    converts::<PinKind>();
    converts::<Placement>();
    converts::<QrRobustness>();
    converts::<ScrollAxis>();
    converts::<ScrollbarSize>();
    converts::<ScrollbarVisibility>();
    converts::<ScrollerControls>();
    converts::<Side>();
    converts::<SidebarSide>();
    converts::<StepLabelPosition>();
    converts::<TabsActivation>();
    converts::<TimePickerVariant>();
    converts::<TimelineAlign>();
    converts::<Variant>();
    converts::<ClassList>();
}
