mod worker;
use bevy::prelude::*;
use worker::DiscordAppWorkerPlugin;
use worker::ThreadboundMessage;

pub struct DiscordAppPlugin;

impl Plugin for DiscordAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DiscordAppWorkerPlugin);

        app.init_resource::<DiscordAppPluginPluginConfig>();
        app.register_type::<DiscordAppPluginPluginConfig>();

        app.add_event::<MuteButtonIdentified>();
        app.register_type::<MuteButtonIdentified>();
        app.add_systems(Update, handle_click);
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct DiscordAppPluginPluginConfig {
    pub refresh_interval: Timer,
}
impl Default for DiscordAppPluginPluginConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Timer::from_seconds(0.01, TimerMode::Repeating),
        }
    }
}

#[derive(Event, Reflect)]
pub struct MuteButtonIdentified {
    pub discord_app: Entity,
    pub pos: IVec2,
}

fn handle_click(
    mut click_events: EventReader<MuteButtonIdentified>,
    mut worker: EventWriter<ThreadboundMessage>,
) {
    for event in click_events.read() {
        info!("User said to investigate position: {:?}", event.pos);
        worker.write(ThreadboundMessage::Lookup { pos: event.pos });
    }
}
