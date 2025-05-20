use crate::DrillId::Unknown;
use crate::ElementInfo;
use bevy::math::IRect;
use bevy::math::IVec2;
use eyre::ensure;
use uiautomation::controls::ControlType::Pane;
use uiautomation::{UIAutomation, UIMatcher};

pub struct DiscordWindowsApp;
impl DiscordWindowsApp {
    pub fn get_sample_element_info() -> ElementInfo {
        ElementInfo {
            name: "#general | Guh-Uh-Guys - Discord".to_string(),
            bounding_rect: IRect {
                min: IVec2::new(
                    3832,
                    568,
                ),
                max: IVec2::new(
                    5768,
                    1624,
                ),
            },
            control_type: Pane,
            localized_control_type: "pane".to_string(),
            class_name: "Chrome_WidgetWin_1".to_string(),
            automation_id: "".to_string(),
            runtime_id: vec![42, 133266].into(),
            drill_id: Unknown,
            children: None,
        }
    }
    pub fn get_matcher(automation: &UIAutomation) -> UIMatcher {
        automation
            .create_matcher()
            .contains_name("Discord")
            .control_type(Pane)
            .classname("Chrome_WidgetWin_1")
    }
    pub fn try_eq(mute_button_element_info: &ElementInfo) -> eyre::Result<()> {
        ensure!(mute_button_element_info.control_type == Pane);
        ensure!(mute_button_element_info.name.contains("Discord"));
        Ok(())
    }
}
impl PartialEq<ElementInfo> for DiscordWindowsApp {
    fn eq(&self, other: &ElementInfo) -> bool {
        DiscordWindowsApp::try_eq(other).is_ok()
    }
}
