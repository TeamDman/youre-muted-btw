use crate::DrillId;
use crate::IntoBevyIRect;
use crate::RuntimeId;
use crate::control_type::YMBControlType;
use crate::update_drill_ids;
use bevy::ecs::component::Component;
use bevy::log::trace;
use bevy::math::IRect;
use bevy::reflect::Reflect;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use uiautomation::UIElement;
use uiautomation::controls::ControlType;

#[derive(Debug, Clone, Reflect, PartialEq, Eq, Serialize, Deserialize, Component)]
#[reflect(no_field_bounds)]
pub struct ElementInfo {
    pub name: String,
    pub bounding_rect: IRect,
    pub control_type: YMBControlType,
    pub localized_control_type: String,
    pub class_name: String,
    pub automation_id: String,
    pub runtime_id: RuntimeId,
    pub drill_id: DrillId,
    pub children: Option<Vec<ElementInfo>>,
}

impl std::fmt::Display for ElementInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}' - {}", self.name, self.drill_id)
    }
}
impl Default for ElementInfo {
    fn default() -> Self {
        ElementInfo {
            name: "UNKNOWN ELEMENT INFO".to_string(),
            bounding_rect: IRect::new(0, 0, 0, 0),
            control_type: ControlType::Pane.into(),
            localized_control_type: "".to_string(),
            class_name: "".to_string(),
            automation_id: "".to_string(),
            runtime_id: RuntimeId::default(),
            drill_id: DrillId::Unknown,
            children: None,
        }
    }
}
impl TryFrom<UIElement> for ElementInfo {
    type Error = uiautomation::Error;
    fn try_from(value: UIElement) -> Result<Self, Self::Error> {
        let name = value.get_name()?;
        let bounding_rect = value.get_bounding_rectangle()?.to_bevy_irect();
        let control_type = value.get_control_type()?;
        let localized_control_type = value.get_localized_control_type()?;
        let class_name = value.get_classname()?;
        let automation_id = value.get_automation_id()?;
        let runtime_id = value.get_runtime_id()?;
        Ok(ElementInfo {
            name,
            bounding_rect,
            control_type: control_type.into(),
            localized_control_type,
            class_name,
            automation_id,
            runtime_id: runtime_id.into(),
            drill_id: DrillId::Unknown,
            children: None,
        })
    }
}
impl ElementInfo {
    pub fn is_stupid_size(&self) -> bool {
        self.bounding_rect.width().abs() > 10_000 || self.bounding_rect.height().abs() > 10_000
    }
    pub fn lookup_drill_id(&self, drill_id: DrillId) -> Option<&ElementInfo> {
        // Log info for problem solving
        trace!(
            "Looking in {} for {}, found children {:?}",
            self,
            drill_id,
            self.children.as_ref().map(|c| c
                .iter()
                .map(|x| format!("{} - {}", x.name, x.drill_id))
                .collect_vec())
        );

        // Only child drill IDs are valid search targets
        // Short circuit here if looking for root
        let drill_id_inner = match drill_id {
            DrillId::Path(drill_id_inner) => drill_id_inner,
            DrillId::Root => return Some(self),
            DrillId::Unknown => return None,
        };

        // Base case
        if drill_id_inner.is_empty() {
            return Some(self);
        }

        // Search children
        for child in self.children.as_ref()? {
            // Only child drill IDs are valid search targets
            let DrillId::Path(child_drill_id) = &child.drill_id else {
                continue;
            };

            // If the child lays on our search path
            if child_drill_id.back() == drill_id_inner.front() {
                // Recurse
                return child
                    .lookup_drill_id(DrillId::Path(drill_id_inner.into_iter().skip(1).collect()));
            }
        }
        None
    }

    pub fn lookup_drill_id_mut(&mut self, drill_id: DrillId) -> Option<&mut ElementInfo> {
        // Log info for problem solving
        trace!(
            "Looking in {} for {}, found children {:?}",
            self,
            drill_id,
            self.children.as_ref().map(|c| c
                .iter()
                .map(|x| format!("{} - {}", x.name, x.drill_id))
                .collect_vec())
        );

        // Only child drill IDs are valid search targets
        // Short circuit here if looking for root
        let drill_id_inner = match drill_id {
            DrillId::Path(drill_id_inner) => drill_id_inner,
            DrillId::Root => return Some(self),
            DrillId::Unknown => return None,
        };

        // Base case
        if drill_id_inner.is_empty() {
            return Some(self);
        }

        // Search children
        for child in self.children.as_deref_mut()?.iter_mut() {
            // Only child drill IDs are valid search targets
            let DrillId::Path(child_drill_id) = &child.drill_id else {
                continue;
            };

            // If the child lays on our search path
            if child_drill_id.back() == drill_id_inner.front() {
                // Recurse
                return child.lookup_drill_id_mut(DrillId::Path(
                    drill_id_inner.into_iter().skip(1).collect(),
                ));
            }
        }
        None
    }

    /// Get the direct child this that is an ancestor of the given drill ID.
    pub fn find_first_child(&self, drill_id: &DrillId) -> Option<&ElementInfo> {
        let window_drill_id = match drill_id {
            DrillId::Root => return None,
            DrillId::Unknown => return None,
            DrillId::Path(inner) => inner.iter().take(1).cloned().collect(),
        };
        self.lookup_drill_id(window_drill_id)
    }

    pub fn get_descendents(&self) -> Vec<&ElementInfo> {
        let mut descendents = vec![];
        if let Some(children) = &self.children {
            for child in children {
                descendents.push(child);
                descendents.extend(child.get_descendents());
            }
        }
        descendents
    }

    pub fn as_identifier(&self) -> String {
        format!(
            "{}_{}",
            self.name.replace(' ', "_").to_lowercase(),
            self.class_name.to_lowercase()
        )
    }
    pub fn as_pascal(&self) -> String {
        self.name
            .split_ascii_whitespace()
            .chain(self.class_name.split_ascii_whitespace())
            .map(|chunk| {
                let mut chunk = chunk.chars();
                match chunk.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chunk).collect(),
                }
            })
            .join("")
    }
    pub fn try_update_drill_ids(&mut self) -> eyre::Result<()> {
        update_drill_ids(self)?;
        Ok(())
    }
}
