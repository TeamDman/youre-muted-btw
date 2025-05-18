use std::error::Error;

pub type WindyResult<T, E = WindyReport> = core::result::Result<T, E>;

pub struct WindyReport {
    inner: eyre::Report,
}
impl From<eyre::Report> for WindyReport {
    fn from(report: eyre::Report) -> Self {
        Self { inner: report }
    }
}
impl std::fmt::Display for WindyReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::fmt::Debug for WindyReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl From<windows::core::Error> for WindyReport {
    #[track_caller]
    fn from(error: windows::core::Error) -> Self {
        Self {
            inner: eyre::Report::new(WrappedWindowsError::from(error)),
        }
    }
}
impl From<clap::Error> for WindyReport {
    #[track_caller]
    fn from(error: clap::Error) -> Self {
        Self {
            inner: eyre::Report::new(error),
        }
    }
}
impl Error for WindyReport {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

pub struct WrappedWindowsError {
    inner: windows::core::Error,
}
impl From<windows::core::Error> for WrappedWindowsError {
    fn from(error: windows::core::Error) -> Self {
        Self { inner: error }
    }
}

impl std::error::Error for WrappedWindowsError {}
impl std::fmt::Display for WrappedWindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::fmt::Debug for WrappedWindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
