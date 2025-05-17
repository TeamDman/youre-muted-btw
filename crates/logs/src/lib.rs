use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::util::SubscriberInitExt;

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

struct DualWriter {
    stdout: std::io::Stderr,
    buffer: Arc<Mutex<Vec<u8>>>,
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

pub fn setup_tracing(debug: bool) -> LogBuffer {
    let buffer = Arc::new(Mutex::new(Vec::new()));

    let subscriber = tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(false)
        .with_max_level(match debug {
            true => tracing::level_filters::LevelFilter::DEBUG,
            false => tracing::level_filters::LevelFilter::INFO,
        })
        .with_ansi(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_span_events(FmtSpan::NONE)
        .with_timer(SystemTime)
        .with_writer(DualWriter {
            stdout: std::io::stderr(),
            buffer: buffer.clone(),
        })
        .finish();

    subscriber.init();

    LogBuffer { buffer }
}
