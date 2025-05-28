use bevy::ecs::component::Component;
use bevy::prelude::*;
use eyre::bail;
use uiautomation::UIElement;
use uiautomation::patterns::UITogglePattern;

#[derive(Debug, Reflect, Component, Clone, Hash, Eq, PartialEq)]
pub enum MuteButtonState {
    Muted,
    NotMuted,
}
impl TryFrom<uiautomation::types::ToggleState> for MuteButtonState {
    type Error = eyre::Error;

    fn try_from(value: uiautomation::types::ToggleState) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            uiautomation::types::ToggleState::On => MuteButtonState::Muted,
            uiautomation::types::ToggleState::Off => MuteButtonState::NotMuted,
            uiautomation::types::ToggleState::Indeterminate => {
                bail!("Indeterminate state is not supported")
            }
        })
    }
}

impl TryFrom<&UIElement> for MuteButtonState {
    type Error = eyre::Error;

    fn try_from(value: &UIElement) -> std::result::Result<Self, Self::Error> {
        match value.get_pattern::<UITogglePattern>() {
            Ok(pattern) => Ok(pattern.get_toggle_state()?.try_into()?),
            Err(pattern_error) => match value.get_name() {
                Ok(name) => match name.as_str() {
                    "Mute" => Ok(MuteButtonState::NotMuted),
                    "Unmute" => Ok(MuteButtonState::Muted),
                    name_error => {
                        bail!(
                            "Failed to get TogglePattern ({pattern_error:?}) and found an unexpected name {name_error:?} for the mute button element."
                        );
                    }
                },
                Err(name_error) => {
                    bail!(
                        "Failed to get TogglePattern ({pattern_error:?}) and failed to get name for the mute button element: {name_error:?}"
                    );
                }
            },
        }
    }
}
