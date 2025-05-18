use bevy::math::IVec2;
use uiautomation::UIAutomation;
use uiautomation::UIElement;
use uiautomation::types::Point;

pub fn find_element_at(
    automation: &mut UIAutomation,
    pos: IVec2,
) -> Result<UIElement, uiautomation::Error> {
    automation.element_from_point(Point::new(pos.x, pos.y))
}
