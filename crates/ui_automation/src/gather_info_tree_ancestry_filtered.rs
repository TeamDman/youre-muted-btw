use crate::DrillId;
use crate::ElementInfo;
use crate::gather_tree_filtered;
use crate::gather_ui_ancestors_including_start;
use bevy::log::warn;
use bevy::reflect::Reflect;
use uiautomation::UIAutomation;
use uiautomation::UIElement;

#[derive(Debug, Clone, Reflect)]
pub struct AncestryTree {
    pub tree: ElementInfo,
    pub start: ElementInfo,
}

pub fn gather_ancestry_tree(
    automation: &UIAutomation,
    start_element: UIElement,
) -> eyre::Result<AncestryTree> {
    let walker = automation.create_tree_walker()?;
    let ancestors = gather_ui_ancestors_including_start(&start_element, &walker)?;

    let root_element = ancestors
        .front()
        .ok_or(uiautomation::Error::new(-1, "No root element found"))?
        .clone();

    let ancestry_filter = |element: &UIElement| {
        ancestors
            .iter()
            .any(|ancestor| ancestor.get_runtime_id() == element.get_runtime_id())
    };
    let mut root_info = gather_tree_filtered(&root_element, &walker, &ancestry_filter, 0)?;
    root_info.drill_id = DrillId::Root;
    root_info.try_update_drill_ids()?;

    let start_element_id = start_element.get_runtime_id()?;
    let start_info = match root_info
        .get_descendents()
        .into_iter()
        .chain(std::iter::once(&root_info))
        .find(|info| info.runtime_id == start_element_id)
        .cloned()
    {
        Some(x) => x,
        None => {
            warn!(
                "Start element {:?} (id: {:?}) not found in tree: {:?}",
                start_element,
                start_element.get_runtime_id(),
                root_info
            );
            root_info.clone()
        }
    };
    Ok(AncestryTree {
        tree: root_info,
        start: start_info,
    })
}
