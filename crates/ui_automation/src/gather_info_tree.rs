use crate::ElementInfo;
use crate::gather_tree_ancestry_filtered;
use crate::gather_tree_filtered;
use crate::update_drill_ids;
use uiautomation::Error;
use uiautomation::UIAutomation;
use uiautomation::UIElement;

pub fn gather_info_tree(
    automation: &mut UIAutomation,
    start_element: UIElement,
) -> Result<ElementInfo, Error> {
    // Setup
    let walker = automation.create_tree_walker()?;

    // Get start drill id
    let start_drill_id = gather_tree_ancestry_filtered(automation, start_element.clone())?
        .start_info
        .drill_id;

    // Get unfiltered tree
    let filter = |_: &UIElement| true;
    let mut tree = gather_tree_filtered(&start_element, &walker, &filter, 0)?;

    // Update drill IDs
    update_drill_ids(tree.children.as_mut(), &start_drill_id);
    tree.drill_id = start_drill_id;

    // Return
    Ok(tree)
}
