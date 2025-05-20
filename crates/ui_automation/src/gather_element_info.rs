use crate::DrillId;
use crate::ElementInfo;
use bevy::math::IRect;
use uiautomation::UIElement;

pub fn gather_single_element_info(element: &UIElement) -> Result<ElementInfo, uiautomation::Error> {
    let name = element.get_name()?;
    let bb = element.get_bounding_rectangle()?;
    let class_name = element.get_classname()?;
    let control_type = element.get_control_type()?.into();
    let localized_control_type = element.get_localized_control_type()?;
    let automation_id = element.get_automation_id()?;
    let runtime_id = element.get_runtime_id()?.into();

    let info = ElementInfo {
        name,
        bounding_rect: IRect::new(bb.get_left(), bb.get_top(), bb.get_right(), bb.get_bottom()),
        control_type,
        localized_control_type,
        class_name,
        automation_id,
        runtime_id,
        children: None,
        drill_id: DrillId::Unknown,
    };
    Ok(info)
}
