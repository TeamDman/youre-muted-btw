use bevy::log::DEFAULT_FILTER;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::log::tracing_subscriber;
use bevy::log::tracing_subscriber::EnvFilter;
use bevy::prelude::*;
use crossbeam_channel::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

fn main() {
    // manually configure tracing so that bevy doesn't take the DEBUG override when running from vscode
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::builder().parse_lossy(format!(
            "{},{}",
            Level::INFO,
            DEFAULT_FILTER
        )))
        .init();
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .add_plugins(MyCounterPlugin)
        .run();
}

// #################################

pub struct MyCounterPlugin;

impl Plugin for MyCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkerPlugin {
            config: WorkerConfig::<ThreadboundMessage, GameboundMessage, WorkerState> {
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
    error: &BevyError,
) -> Result<()> {
    reply_tx.send(GameboundMessage::Error {
        message: format!("{error:#?}"),
    })?;
    Ok(())
}

fn handle_threadbound_message(
    msg: &ThreadboundMessage,
    reply_tx: &Sender<GameboundMessage>,
    state: &mut WorkerState,
) -> Result<()> {
    match msg {
        ThreadboundMessage::Reset => {
            state.count = 0;
        }
        ThreadboundMessage::Increment => {
            state.count += 1;
        }
    }
    if state.count == 13 {
        return Err(eyre::eyre!("Count reached 13!").into());
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
            reset_at: 20,
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
    mut error_count: Local<u32>,
) -> Result {
    for msg in snapshot.read() {
        match msg {
            GameboundMessage::Latest(x) => {
                counter_state.count = *x;
                if *x >= counter_state.reset_at {
                    threadbound_messages.write(ThreadboundMessage::Reset);
                }
            }
            GameboundMessage::Error { message } => {
                *error_count += 1;
                if *error_count > 3 {
                    return Err(BevyError::from(message.clone()));
                } else {
                    error!("Error from worker: {message:#?}");
                }
            }
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
