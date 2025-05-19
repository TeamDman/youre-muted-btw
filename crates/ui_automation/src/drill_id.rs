use std::collections::VecDeque;

use bevy::reflect::Reflect;
use eyre::bail;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use uiautomation::UIAutomation;
use uiautomation::UIElement;

use crate::{gather_single_element_info, Drillable, ElementInfo};
#[derive(Debug, Eq, PartialEq, Clone, Reflect, Default, Hash, Serialize, Deserialize)]
pub enum DrillId {
    Root,
    Path(VecDeque<usize>),
    #[default]
    Unknown,
}
impl std::fmt::Display for DrillId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrillId::Root => write!(f, "DrillId::Root"),
            DrillId::Path(drill_id) => write!(
                f,
                "DrillId::Child([{}].into())",
                drill_id
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            DrillId::Unknown => write!(f, "DrillId::Unknown"),
        }
    }
}

impl FromIterator<usize> for DrillId {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        DrillId::Path(iter.into_iter().collect())
    }
}
impl<T> From<T> for DrillId
where
    T: IntoIterator<Item = usize>,
{
    fn from(value: T) -> Self {
        DrillId::Path(value.into_iter().collect())
    }
}
impl DrillId {
    pub fn try_join(&self, other: impl Into<DrillId>) -> eyre::Result<DrillId> {
        match (self, other.into()) {
            (DrillId::Unknown, x) => bail!("Cannot join an unknown drill ID with {x:?}"),
            (x, DrillId::Unknown) => bail!("Cannot join {x:?} with an unknown drill ID"),
            (x, DrillId::Root) => {
                bail!("Cannot join {x:?} with a root drill ID")
            }
            (DrillId::Root, DrillId::Path(path)) => Ok(DrillId::Path(path)),
            (DrillId::Path(parent), DrillId::Path(rhs)) => {
                let mut new = parent.clone();
                new.extend(rhs);
                Ok(DrillId::Path(new))
            }
        }
    }
    pub fn display_highlighted_index(&self, index: usize) -> String {
        match self {
            DrillId::Root => "DrillId::Root".to_string(),
            DrillId::Path(path) => {
                format!(
                    "DrillId::Child([{}])",
                    path.iter()
                        .enumerate()
                        .map(|(i, x)| if i == index {
                            format!(">>{x}<<")
                        } else {
                            x.to_string()
                        })
                        .collect_vec()
                        .join(", ")
                )
            }
            DrillId::Unknown => "DrillId::Unknown".to_string(),
        }
    }
    pub fn resolve(self) -> eyre::Result<VecDeque<(UIElement, ElementInfo)>> {
        let automation = UIAutomation::new()?;
        let walker = automation.create_tree_walker()?;
        let root = automation.get_root_element()?;
        let mut root_info = gather_single_element_info(&root)?;
        root_info.drill_id = DrillId::Root;
        let mut children = root.clone().drill(&walker, self)?;
        children.push_front((root, root_info));
        Ok(children)
    }
}
