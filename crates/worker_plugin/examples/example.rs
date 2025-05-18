use bevy::log::LogPlugin;
use bevy::prelude::*;
use crossbeam_channel::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

pub struct MyCounterPlugin;

impl Plugin for MyCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkerPlugin {
            config: WorkerConfig::<ThreadboundMessage, GameboundMessage, WorkerState, _, _, _> {
                name: "MyWorker".to_string(),
                handle_threadbound_message,
                handle_threadbound_message_error_handler,
                ..default()
            },
        });
        app.init_resource::<CounterState>();
        app.add_systems(Update, handle_gamebound_messages);
        app.add_systems(Update, tick_worker);
        app.add_systems(Update, print_count);
    }
}

// #################################

#[derive(Default)]
pub struct WorkerState {
    pub count: u32,
}

#[derive(Debug, Reflect, Clone, Event)]
pub enum ThreadboundMessage {
    Increment,
    Reset,
}

fn handle_threadbound_message_error_handler(
    _msg: &ThreadboundMessage,
    reply_tx: &Sender<GameboundMessage>,
    _state: &mut WorkerState,
    error: &eyre::Error,
) -> eyre::Result<()> {
    reply_tx.send(GameboundMessage::Error {
        message: format!("{error:#?}"),
    })?;
    Ok(())
}

fn handle_threadbound_message(
    msg: &ThreadboundMessage,
    reply_tx: &Sender<GameboundMessage>,
    state: &mut WorkerState,
) -> eyre::Result<()> {
    match msg {
        ThreadboundMessage::Reset => {
            state.count = 0;
        }
        ThreadboundMessage::Increment => {
            state.count += 1;
        }
    }
    reply_tx.send(GameboundMessage::Latest(state.count))?;
    Ok(())
}

// #################################

#[derive(Debug, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct CounterState {
    count: u32,
    reset_at: u32,
    tick_timer: Timer,
    print_timer: Timer,
}
impl Default for CounterState {
    fn default() -> Self {
        Self {
            count: 0,
            reset_at: 100,
            tick_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            print_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Reflect, Clone, Event)]
pub enum GameboundMessage {
    Latest(u32),
    Error { message: String },
}

fn handle_gamebound_messages(
    mut snapshot: EventReader<GameboundMessage>,
    mut counter_state: ResMut<CounterState>,
    mut threadbound_messages: EventWriter<ThreadboundMessage>,
) -> Result {
    for msg in snapshot.read() {
        match msg {
            GameboundMessage::Latest(x) => {
                counter_state.count = *x;
                if *x >= counter_state.reset_at {
                    threadbound_messages.write(ThreadboundMessage::Reset);
                }
            }
            GameboundMessage::Error { message } => return Err(BevyError::from(message.clone())),
        }
    }
    Ok(())
}

// #################################

fn tick_worker(
    mut threadbound_messages: EventWriter<ThreadboundMessage>,
    mut counter_state: ResMut<CounterState>,
    time: Res<Time>,
) {
    if !counter_state.tick_timer.tick(time.delta()).just_finished() {
        return;
    }
    threadbound_messages.write(ThreadboundMessage::Increment);
}

fn print_count(mut counter_state: ResMut<CounterState>, time: Res<Time>) {
    if !counter_state.print_timer.tick(time.delta()).just_finished() {
        return;
    }
    info!("Current count: {}", counter_state.count);
}

// #################################

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::INFO,
            ..default()
        }))
        .add_plugins(MyCounterPlugin)
        .run();
}
