use crate::DrillId;
use crate::ElementInfo;
use crate::StopBehaviour;
use crate::gather_children;
use crate::gather_single_element_info;

pub fn gather_root() -> eyre::Result<ElementInfo> {
    let automation = uiautomation::UIAutomation::new()?;
    let root_element = automation.get_root_element()?;
    let walker = automation.create_tree_walker()?;
    let children = gather_children(&walker, &root_element, &StopBehaviour::RootEndEncountered);
    let mut root_element_info = gather_single_element_info(&root_element)?;
    root_element_info.drill_id = DrillId::Root;
    let mut new_children = Vec::new();
    for child in children {
        let child_info = gather_single_element_info(&child)?;
        new_children.push(child_info);
    }
    root_element_info.children = Some(new_children);
    root_element_info.try_update_drill_ids()?;
    Ok(root_element_info)
}
