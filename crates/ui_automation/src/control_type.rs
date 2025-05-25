use bevy::reflect::Reflect;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::inspector_egui_impls::InspectorPrimitive;
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use serde::Deserialize;
use serde::Serialize;
use uiautomation::controls::ControlType;

use serde::de::Error as DeError;

#[derive(Reflect, Clone, PartialEq, Eq, Serialize)]
pub struct YMBControlType {
    inner: i32,
}
impl std::fmt::Display for YMBControlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_uia_control_type())
    }
}
impl std::fmt::Debug for YMBControlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_uia_control_type())
    }
}
impl YMBControlType {
    pub fn as_uia_control_type(&self) -> ControlType {
        ControlType::try_from(self.inner).unwrap()
    }
}
impl Default for YMBControlType {
    fn default() -> Self {
        ControlType::Pane.into()
    }
}
impl From<ControlType> for YMBControlType {
    fn from(control_type: ControlType) -> Self {
        Self {
            inner: control_type as i32,
        }
    }
}
impl TryFrom<YMBControlType> for ControlType {
    type Error = uiautomation::Error;
    fn try_from(value: YMBControlType) -> Result<Self, Self::Error> {
        ControlType::try_from(value.inner)
    }
}
impl<'de> Deserialize<'de> for YMBControlType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = i32::deserialize(deserializer)?;
        let control_type = ControlType::try_from(inner)
            .map_err(|_| D::Error::custom(format!("Invalid ControlType value: {}", inner)))?;
        Ok(control_type.into())
    }
}

impl InspectorPrimitive for YMBControlType {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        id: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;
        let mut current_selection = self.as_uia_control_type();

        // Define all ControlType variants for the dropdown.
        // Ideally, this would come from an iterator if the uiautomation crate provided one.
        const ALL_CONTROL_TYPES: [ControlType; 41] = [
            ControlType::Button,
            ControlType::Calendar,
            ControlType::CheckBox,
            ControlType::ComboBox,
            ControlType::Edit,
            ControlType::Hyperlink,
            ControlType::Image,
            ControlType::ListItem,
            ControlType::List,
            ControlType::Menu,
            ControlType::MenuBar,
            ControlType::MenuItem,
            ControlType::ProgressBar,
            ControlType::RadioButton,
            ControlType::ScrollBar,
            ControlType::Slider,
            ControlType::Spinner,
            ControlType::StatusBar,
            ControlType::Tab,
            ControlType::TabItem,
            ControlType::Text,
            ControlType::ToolBar,
            ControlType::ToolTip,
            ControlType::Tree,
            ControlType::TreeItem,
            ControlType::Custom,
            ControlType::Group,
            ControlType::Thumb,
            ControlType::DataGrid,
            ControlType::DataItem,
            ControlType::Document,
            ControlType::SplitButton,
            ControlType::Window,
            ControlType::Pane,
            ControlType::Header,
            ControlType::HeaderItem,
            ControlType::Table,
            ControlType::TitleBar,
            ControlType::Separator,
            ControlType::SemanticZoom,
            ControlType::AppBar,
        ];

        egui::ComboBox::from_id_salt(id.with("ymb_control_type_dropdown"))
            .selected_text(format!("{:?}", current_selection))
            .show_ui(ui, |ui| {
                for variant in ALL_CONTROL_TYPES.iter() {
                    if ui
                        .selectable_value(
                            &mut current_selection,
                            *variant,
                            format!("{:?}", variant),
                        )
                        .clicked()
                    {
                        if self.inner != (*variant as i32) {
                            self.inner = *variant as i32;
                            changed = true;
                        }
                    }
                }
            });

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        ui.label(format!("{:?}", self.as_uia_control_type()));
    }
}
