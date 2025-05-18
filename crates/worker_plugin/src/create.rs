use crate::Bridge;
use crate::WorkerConfig;
use crate::WorkerMessage;
use crate::WorkerStateTrait;
use bevy::prelude::*;
pub use crossbeam_channel::Receiver;
pub use crossbeam_channel::Sender;
use crossbeam_channel::bounded;
use std::thread;
use windows::Win32::System::Com::COINIT_MULTITHREADED;
use windows::Win32::System::Com::CoInitializeEx;

pub fn create_worker_thread<
    ThreadboundMessage,
    GameboundMessage,
    WorkerState,
>(
    config: Res<
        WorkerConfig<
            ThreadboundMessage,
            GameboundMessage,
            WorkerState,
        >,
    >,
    mut commands: Commands,
) where
    ThreadboundMessage: WorkerMessage,
    GameboundMessage: WorkerMessage,
    WorkerState: WorkerStateTrait,
{
    let (game_tx, game_rx) = bounded::<GameboundMessage>(config.gamebound_channel_capacity);
    let (thread_tx, thread_rx) = bounded::<ThreadboundMessage>(config.threadbound_channel_capacity);

    commands.insert_resource(Bridge {
        sender: thread_tx,
        receiver: game_rx,
    });

    let name = config.name.clone();
    let handler = config.handle_threadbound_message;
    let handler_error_handler = config.handle_threadbound_message_error_handler;
    let sleep_duration = config.sleep_duration;
    let is_ui_automation_thread = config.is_ui_automation_thread;
    let receiver = config.threadbound_message_receiver;
    if let Err(e) = thread::Builder::new().name(name.clone()).spawn(move || {
        if is_ui_automation_thread {
            unsafe {
                // Initialize COM in MTA mode
                // https://learn.microsoft.com/en-us/dotnet/framework/ui-automation/ui-automation-threading-issues
                // https://learn.microsoft.com/en-us/windows/win32/com/multithreaded-apartments
                if let Err(e) = CoInitializeEx(None, COINIT_MULTITHREADED).ok() {
                    error!("[{}] Failed to initialize COM: {:?}", name, e);
                }
                debug!("[{}] COM initialized in MTA mode.", name);
            }
        }

        let Ok(mut state) = WorkerState::try_default() else {
            error!("[{}] Failed to initialize state", name);
            return;
        };

        loop {
            let msg = match (receiver)(&thread_rx, &mut state) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("[{}] Threadbound channel receiver failure: {:?}, quitting loop", name, e);
                    break;
                }
            };
            if let Err(e) = (handler)(&msg, &game_tx, &mut state) {
                // TODO: leave logging the error to the handler
                error!(
                    "[{}] Failed to process thread message {:?}, got error {:?}",
                    name, msg, e
                );
                if let Err(ee) = (handler_error_handler)(&msg, &game_tx, &mut state, &e) {
                    error!(
                        "[{}] BAD NEWS! Failed while processing error handler for message {:?} that produced error {:?}, got new error {:?}",
                        name, msg, e, ee
                    );
                }
            }
            std::thread::sleep(sleep_duration);
        }
    }) {
        error!("[{}] Failed to spawn thread: {:?}", config.name, e);
    } else {
        info!("[{}] Thread created", config.name);
    }
}
