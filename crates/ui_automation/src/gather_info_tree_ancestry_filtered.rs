use crate::DrillId;
use crate::ElementInfo;
use crate::gather_tree_filtered;
use crate::gather_ui_ancestors_including_start;
use crate::update_drill_ids;
use uiautomation::UIAutomation;
use uiautomation::UIElement;

#[derive(Debug, Clone)]
pub struct GatheredTree {
    pub ui_tree: ElementInfo,
    pub start_info: ElementInfo,
}

pub fn gather_tree_ancestry_filtered(
    automation: &mut UIAutomation,
    start_element: UIElement,
) -> Result<GatheredTree, uiautomation::Error> {
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

    update_drill_ids(root_info.children.as_mut(), &DrillId::Root);

    let start_info = root_info
        .get_descendents()
        .into_iter()
        .chain(std::iter::once(&root_info))
        .find(|info| match start_element.get_runtime_id() {
            Ok(id) => info.runtime_id == id,
            Err(_) => false,
        })
        .cloned();
    let Some(start_info) = start_info else {
        return Err(uiautomation::Error::new(
            -1,
            format!(
                "Start element {:?} (id: {:?}) not found in tree: {:?}",
                start_element,
                start_element.get_runtime_id(),
                root_info
            )
            .as_str(),
        ));
    };
    Ok(GatheredTree {
        ui_tree: root_info,
        start_info,
    })
}
