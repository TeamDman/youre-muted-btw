use bevy::prelude::*;
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use std::any::type_name;
use std::time::Duration;
use uiautomation::UIAutomation;
use uiautomation::UIElement;
use ymb_ui_automation::AncestryTree;
use ymb_ui_automation::DiscordMuteButton;
use ymb_ui_automation::ElementInfo;
use ymb_ui_automation::MuteButtonState;
use ymb_ui_automation::YMBControlType;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;
use ymb_worker_plugin::WorkerStateTrait;

pub struct UIAutomationPlugin;

impl Plugin for UIAutomationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkerPlugin {
            config: WorkerConfig::<
                UIWorkerThreadboundMessage,
                UIWorkerGameboundMessage,
                UIWorkerState,
            > {
                name: "ElementInfoPluginWorker".to_string(),
                handle_threadbound_message,
                threadbound_message_receiver: |thread_rx, _state| {
                    if thread_rx.is_empty() {
                        thread_rx.recv().map_err(BevyError::from)
                    } else {
                        Ok(thread_rx.try_iter().last().unwrap())
                    }
                },
                ..default()
            },
        });
        app.add_systems(Update, handle_gamebound_messages);
        app.add_systems(Update, tick_worker);
        app.add_systems(Startup, startup_fetch);
        app.init_resource::<UIAutomationPluginConfig>();
        app.register_type::<UIAutomationPluginConfig>();
        app.register_type::<ElementInfo>();
        app.register_type::<AncestryTree>();
        app.register_type::<YMBControlType>();
        app.register_type::<MuteButtonState>();
        app.register_type::<DiscordMuteButton>();
        app.register_type_data::<YMBControlType, InspectorEguiImpl>();
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct UIAutomationPluginConfig {
    pub refresh_interval: Timer,
}

impl Default for UIAutomationPluginConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
        }
    }
}

pub struct UIWorkerState {
    automation: UIAutomation,
    mute_button: Option<UIElement>,
}
impl WorkerStateTrait for UIWorkerState {
    type Error = BevyError;

    fn try_default() -> std::result::Result<Self, Self::Error> {
        let automation = UIAutomation::new()?;
        Ok(Self {
            automation,
            mute_button: None,
        })
    }
}

#[derive(Debug, Reflect, Clone, Event)]
pub enum UIWorkerThreadboundMessage {
    DetectMuteButtonState,
}

#[derive(Debug, Reflect, Clone, Event)]
#[reflect(from_reflect = false)]
pub enum UIWorkerGameboundMessage {
    MuteButtonObserved { state: MuteButtonState },
}

fn handle_threadbound_message(
    msg: &UIWorkerThreadboundMessage,
    reply_tx: &Sender<UIWorkerGameboundMessage>,
    state: &mut UIWorkerState,
) -> Result<()> {
    info!("Handling threadbound message: {:?}", msg);
    match msg {
        UIWorkerThreadboundMessage::DetectMuteButtonState => {
            if state.mute_button.is_none() {
                if let Ok(element) = DiscordMuteButton::try_find(&state.automation) {
                    info!("Found mute button element: {:?}", element);
                    state.mute_button = Some(element);
                }
            }
            if let Some(mute_button) = &state.mute_button {
                match MuteButtonState::try_from(mute_button) {
                    Ok(state) => {
                        reply_tx.send(UIWorkerGameboundMessage::MuteButtonObserved { state })?;
                    }
                    Err(x) => {
                        warn!("Failed to get toggle state from mute button: {:?}", x);
                        state.mute_button = None; // Reset the mute button if an error occurs
                        reply_tx.send(UIWorkerGameboundMessage::MuteButtonObserved {
                            state: MuteButtonState::NotMuted, // Default to Off if error occurs
                        })?;
                    }
                }
            } else {
                warn!("Mute button not found.");
            }
        }
    }
    info!("Threadbound message handled successfully.");
    Ok(())
}

fn handle_gamebound_messages(
    mut messages: EventReader<UIWorkerGameboundMessage>,
    mut mute_button: Query<&mut MuteButtonState, With<DiscordMuteButton>>,
    mut commands: Commands,
) -> Result {
    for msg in messages.read() {
        match msg {
            UIWorkerGameboundMessage::MuteButtonObserved { state } => {
                info!("Received mute button state: {:?}", state);
                if let Ok(mut toggle_state) = mute_button.single_mut() {
                    *toggle_state = state.clone();
                    info!("Updated toggle state: {:?}", toggle_state);
                } else {
                    info!("Spawning new DiscordMuteButton with state: {:?}", state);
                    commands.spawn((
                        DiscordMuteButton,
                        state.clone(),
                        Name::new(type_name::<DiscordMuteButton>()),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn tick_worker(
    mut threadbound_messages: EventWriter<UIWorkerThreadboundMessage>,
    mut config: ResMut<UIAutomationPluginConfig>,
    time: Res<Time>,
) {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return;
    }
    threadbound_messages.write(UIWorkerThreadboundMessage::DetectMuteButtonState);
}

fn startup_fetch(mut threadbound_messages: EventWriter<UIWorkerThreadboundMessage>) {
    threadbound_messages.write(UIWorkerThreadboundMessage::DetectMuteButtonState);
}
