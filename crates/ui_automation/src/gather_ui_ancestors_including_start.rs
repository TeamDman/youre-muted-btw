use std::collections::VecDeque;

use uiautomation::UIElement;
use uiautomation::UITreeWalker;

pub fn gather_ui_ancestors_including_start(
    element: &UIElement,
    walker: &UITreeWalker,
) -> Result<VecDeque<UIElement>, uiautomation::Error> {
    let mut ancestors = VecDeque::new();
    let mut current_element = Some(element.clone());
    while let Some(elem) = current_element {
        ancestors.push_front(elem.clone());
        current_element = walker.get_parent(&elem).ok();
    }
    Ok(ancestors)
}
