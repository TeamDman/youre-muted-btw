use bevy::prelude::*;
use interprocess::os::windows::named_pipe::PipeListener;
use interprocess::os::windows::named_pipe::PipeListenerOptions;
use interprocess::os::windows::named_pipe::pipe_mode;
use std::io::Read;
use std::io::{self};
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

pub struct IpcPlugin;

impl Plugin for IpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IpcWorkerThreadboundMessage>();
        app.register_type::<IpcWorkerGameboundMessage>();
        app.add_plugins(WorkerPlugin::<
            IpcWorkerThreadboundMessage,
            IpcWorkerGameboundMessage,
            IpcWorkerState,
        > {
            config: WorkerConfig {
                name: "IpcWorker".to_string(),
                is_ui_automation_thread: false,
                handle_threadbound_message,
                threadbound_message_receiver: |thread_rx, _state| {
                    thread_rx.recv().map_err(bevy::prelude::BevyError::from)
                },
                ..default()
            },
        });
        app.add_systems(Startup, setup_ipc_and_worker);
        app.add_systems(Update, handle_gamebound_messages);
    }
}

#[derive(Debug, Clone, Reflect, Event)]
pub enum IpcWorkerThreadboundMessage {
    InitAndListen,
}

#[derive(Debug, Clone, Reflect, Event)]
pub enum BevyboundIPCMessage {
    ToggleWindowVisibility,
    DebugMessageReceived(String),
}

#[derive(Debug, Clone, Reflect, Event)]
pub enum IpcWorkerGameboundMessage {
    MessageReceived(BevyboundIPCMessage),
}

#[derive(Default)]
pub struct IpcWorkerState {
    listener: Option<PipeListener<pipe_mode::Bytes, pipe_mode::Bytes>>,
}

fn handle_threadbound_message(
    msg: &IpcWorkerThreadboundMessage,
    reply_tx: &Sender<IpcWorkerGameboundMessage>,
    state: &mut IpcWorkerState,
) -> Result<(), bevy::prelude::BevyError> {
    info!("IpcWorker: Received threadbound message: {:?}", msg);
    match msg {
        IpcWorkerThreadboundMessage::InitAndListen => {
            if state.listener.is_some() {
                warn!("IpcWorker: Listener already initialized. Ignoring InitAndListen message.");
                return Ok(());
            }
            let pipe_name = std::env::var("YMB_IPC_PIPE_NAME").map_err(bevy::prelude::BevyError::from)?;
            info!("IpcWorker: Initializing listener for pipe: {}", pipe_name);
            let listener = PipeListenerOptions::new()
                .path(pipe_name.clone())
                .create_duplex::<pipe_mode::Bytes>()
                .map_err(bevy::prelude::BevyError::from)?;
            state.listener = Some(listener);
            info!("IpcWorker: Starting listener accept loop.");
            let listener_ref = state.listener.as_ref().unwrap();
            for incoming_conn in listener_ref.incoming() {
                match incoming_conn {
                    Ok(mut stream) => {
                        debug!("IpcWorker: Accepted new connection.");
                        let mut buffer = String::new();
                        match stream.read_to_string(&mut buffer) {
                            Ok(_) => {
                                let msg = buffer.trim();
                                info!("IpcWorker: Received message: '{}'", msg);
                                if msg == "ToggleWindowVisibility" {
                                    reply_tx.send(IpcWorkerGameboundMessage::MessageReceived(BevyboundIPCMessage::ToggleWindowVisibility))?;
                                } else {
                                    reply_tx.send(IpcWorkerGameboundMessage::MessageReceived(BevyboundIPCMessage::DebugMessageReceived(msg.to_string())))?;
                                }
                            }
                            Err(e) => {
                                error!("IpcWorker: Failed to read from stream: {}", e);
                                if e.kind() == io::ErrorKind::BrokenPipe {
                                    continue;
                                }
                                return Err(bevy::prelude::BevyError::from(e));
                            }
                        }
                    }
                    Err(e) => {
                        error!("IpcWorker: Failed to accept connection: {}", e);
                        return Err(bevy::prelude::BevyError::from(e));
                    }
                }
            }
            info!("IpcWorker: Listener accept loop ended.");
        }
    }
    Ok(())
}

fn setup_ipc_and_worker(mut ipc_worker_events: EventWriter<IpcWorkerThreadboundMessage>) {
    // Do not read the pipe name here; let the worker thread read the env var
    ipc_worker_events.write(IpcWorkerThreadboundMessage::InitAndListen);
}

fn handle_gamebound_messages(
    mut messages: EventReader<IpcWorkerGameboundMessage>,
) {
    for msg in messages.read() {
        match msg {
            IpcWorkerGameboundMessage::MessageReceived(BevyboundIPCMessage::DebugMessageReceived(text)) => {
                debug!("Bevy App (Main Thread): Received IPC Debug Message: '{}'", text);
            }
            IpcWorkerGameboundMessage::MessageReceived(BevyboundIPCMessage::ToggleWindowVisibility) => {
                info!("Received ToggleWindowVisibility IPC message (no-op in ipc_plugin)");
                // This plugin should not handle window logic directly.
            }
        }
    }
}
