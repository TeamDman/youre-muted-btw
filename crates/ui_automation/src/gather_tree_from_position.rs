use bevy::math::IVec2;
use uiautomation::UIAutomation;

use crate::GatheredTree;
use crate::find_element_at;
use crate::gather_tree_ancestry_filtered;

pub fn gather_tree_from_position(pos: IVec2) -> Result<GatheredTree, uiautomation::Error> {
    let mut automation = UIAutomation::new()?;
    let start = find_element_at(&mut automation, pos)?;
    let gathered = gather_tree_ancestry_filtered(&mut automation, start)?;
    Ok(gathered)
}
