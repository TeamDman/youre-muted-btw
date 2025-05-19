use bevy::math::IVec2;
use uiautomation::UIAutomation;

use crate::AncestryTree;
use crate::find_element_at;
use crate::gather_ancestry_tree;

pub fn gather_tree_from_position(pos: IVec2) -> eyre::Result<AncestryTree> {
    let automation = UIAutomation::new()?;
    let start = find_element_at(&automation, pos)?;
    let gathered = gather_ancestry_tree(&automation, start)?;
    Ok(gathered)
}
