use crate::ElementInfo;
use bevy::math::IRect;
use bevy::math::IVec2;
use eyre::ensure;
use uiautomation::UIAutomation;
use uiautomation::UIMatcher;
use uiautomation::controls::ControlType::Button;

pub struct DiscordMuteButton;
impl DiscordMuteButton {
    pub fn get_sample_element_info() -> ElementInfo {
        ElementInfo {
            name: "Mute".to_string(),
            bounding_rect: IRect {
                min: IVec2::new(4158, 1550),
                max: IVec2::new(4199, 1590),
            },
            control_type: Button.into(),
            localized_control_type: "button".to_string(),
            class_name: "".to_string(),
            automation_id: "".to_string(),
            runtime_id: vec![42, 788570, 4, 4294966141u32].into(),
            drill_id: [0, 0, 1, 0, 1, 0, 0, 0, 1, 3, 1, 1].into(),
            children: None,
        }
    }
    pub fn get_matcher(automation: &UIAutomation) -> UIMatcher {
        automation
            .create_matcher()
            .name("Mute")
            .control_type(Button)
            .classname("")
    }
    pub fn try_eq(mute_button_element_info: &ElementInfo) -> eyre::Result<()> {
        ensure!(mute_button_element_info.control_type == Button.into());
        ensure!(mute_button_element_info.name == "Mute");
        let rect = mute_button_element_info.bounding_rect;
        let width = rect.width() as f64;
        let height = rect.height() as f64;
        let ratio = width / height;
        let threshold = 0.2;
        let lower_bound = 1.0 - threshold;
        let upper_bound = 1.0 + threshold;
        ensure!(
            ratio > lower_bound && ratio < upper_bound,
            "Expected a width/height ratio between 0.8 and 1.1, got {ratio}"
        );
        Ok(())
    }
}
impl PartialEq<ElementInfo> for DiscordMuteButton {
    fn eq(&self, other: &ElementInfo) -> bool {
        DiscordMuteButton::try_eq(other).is_ok()
    }
}
