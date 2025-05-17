use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::debug;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::SubscriberBuilder;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::util::SubscriberInitExt;
use ymb_args::GlobalArgs;

/// Logs are stored in a buffer to be displayed in the console when the user clicks show logs
#[derive(Debug, Clone, Default)]
pub struct LogBuffer {
    buffer: Arc<Mutex<Vec<u8>>>,
}
impl Deref for LogBuffer {
    type Target = Arc<Mutex<Vec<u8>>>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl DerefMut for LogBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct DualWriter {
    pub stdout: std::io::Stderr,
    pub buffer: LogBuffer,
}
impl DualWriter {
    pub fn new() -> Self {
        DualWriter {
            stdout: std::io::stderr(),
            buffer: LogBuffer::default(),
        }
    }
}

impl Write for DualWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Write to stdout
        self.stdout.write(buf)?;

        // Write to buffer
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }
}

impl<'a> MakeWriter<'a> for DualWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        DualWriter {
            stdout: std::io::stderr(),
            buffer: self.buffer.clone(),
        }
    }
}

pub fn setup_tracing(
    global_args: &GlobalArgs,
    writer: impl for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
) -> eyre::Result<()> {
    let mine = SubscriberBuilder::default()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(false)
        .with_ansi(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_span_events(FmtSpan::NONE)
        .with_timer(SystemTime)
        .with_writer(writer)
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::builder().parse_lossy(format!(
                "{default_log_level},{bevy_defaults}",
                default_log_level = match global_args.debug {
                    true => LevelFilter::DEBUG,
                    false => LevelFilter::INFO,
                },
                bevy_defaults = bevy_log::DEFAULT_FILTER
            ))
        }));
    let subscriber = mine.finish();
    subscriber.init();
    ahoy(global_args);
    Ok(())
}

fn ahoy(global_args: &GlobalArgs) {
    info!("");
    info!("    X                                                                ");
    info!("    XX         X          X            XXXXXXXX        X            X");
    info!("   XXXX        X          X        XXXX        X       XX          XX");
    info!("  XX  X        X          X      XXX            X       XX         X ");
    info!("  X   XX       X          XX    XX              XX       XX       XX ");
    info!(" X     X       XXXXXXXXXXXXX    X                X         XXXXXXX   ");
    info!("XXXXXXXXX      X           X    X               XX            X      ");
    info!("X       X      X           X    X               X             X      ");
    info!("X         X     X           XX   XX             X             XX     ");
    info!("X         X     X            X    XXX        XXX              X      ");
    info!("X           X    X            X      XXXXXXXXX               XX      ");
    info!("                                                            XX       ");
    info!("                                                           XX        ");
    debug!("Debug logging enabled: {}", global_args.debug);
}
