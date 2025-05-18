use itertools::Itertools;
use uiautomation::{UIElement, UITreeWalker};

use crate::{gather_single_element_info, ElementInfo, GatherChildrenable, StopBehaviour};

pub fn gather_tree_filtered(
    element: &UIElement,
    walker: &UITreeWalker,
    filter: &dyn Fn(&UIElement) -> bool,
    depth: usize,
) -> Result<ElementInfo, uiautomation::Error> {
    let mut element_info = gather_single_element_info(element)?;
    if filter(element) {
        let children = element
            .gather_children(
                walker,
                if depth == 0 {
                    &StopBehaviour::RootEndEncountered
                } else {
                    &StopBehaviour::EndOfSiblings
                },
            )
            .into_iter()
            .enumerate()
            .filter_map(|(i, child)| {
                if filter(&child) {
                    gather_tree_filtered(&child, walker, filter, depth + 1).ok()
                } else {
                    gather_single_element_info(&child).ok()
                }
                .map(|mut child_info| {
                    child_info.drill_id = vec![i].into();
                    child_info
                })
            })
            .collect_vec();

        element_info.children = Some(children);
    }

    Ok(element_info)
}
