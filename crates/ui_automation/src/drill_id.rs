use std::collections::VecDeque;

use bevy::reflect::Reflect;
use serde::Deserialize;
use serde::Serialize;
use uiautomation::UIAutomation;
use uiautomation::UIElement;

use crate::Drillable;
#[derive(Debug, Eq, PartialEq, Clone, Reflect, Default, Hash, Serialize, Deserialize)]
pub enum DrillId {
    Root,
    Child(VecDeque<usize>),
    #[default]
    Unknown,
}
impl DrillId {
    pub fn as_child(&self) -> Option<&VecDeque<usize>> {
        match self {
            DrillId::Child(child) => Some(child),
            _ => None,
        }
    }

    /// Given two absolute drill IDs, returns the path to this drill ID starting from the other drill ID.
    pub fn relative_to(&self, parent: &DrillId) -> DrillId {
        let (self_iter, mark_iter) = match (self, parent) {
            (DrillId::Root, DrillId::Root) => return DrillId::Root,
            (DrillId::Root, _) => return DrillId::Unknown,
            (_, DrillId::Root) => return self.clone(),
            (DrillId::Child(self_iter), DrillId::Child(mark_iter)) => (self_iter, mark_iter),
            _ => return DrillId::Unknown,
        };

        // this should have the parent as a prefix
        // if not, return unknown
        for x in mark_iter.iter().zip(self_iter.iter()) {
            if x.0 != x.1 {
                return DrillId::Unknown;
            }
        }

        // if len is the same, return root
        if self_iter.len() == mark_iter.len() {
            return DrillId::Root;
        }

        // return the part of the path that is not in the parent
        DrillId::Child(
            self_iter
                .clone()
                .into_iter()
                .skip(mark_iter.len())
                .collect(),
        )
    }
}
impl FromIterator<usize> for DrillId {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        DrillId::Child(iter.into_iter().collect())
    }
}
impl From<Vec<usize>> for DrillId {
    fn from(value: Vec<usize>) -> Self {
        DrillId::Child(value.into())
    }
}
impl From<VecDeque<usize>> for DrillId {
    fn from(value: VecDeque<usize>) -> Self {
        DrillId::Child(value)
    }
}
impl From<Vec<i32>> for DrillId {
    fn from(value: Vec<i32>) -> Self {
        DrillId::Child(value.into_iter().map(|x| x as usize).collect())
    }
}
impl From<VecDeque<i32>> for DrillId {
    fn from(value: VecDeque<i32>) -> Self {
        DrillId::Child(value.into_iter().map(|x| x as usize).collect())
    }
}
impl std::fmt::Display for DrillId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrillId::Root => write!(f, "DrillId::Root"),
            DrillId::Child(drill_id) => write!(
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

impl DrillId {
    pub fn resolve(self) -> eyre::Result<UIElement> {
        let automation = UIAutomation::new()?;
        let walker = automation.create_tree_walker()?;
        let root = automation.get_root_element()?;
        let elem = root.drill(&walker, self)?;
        Ok(elem)
    }
}
