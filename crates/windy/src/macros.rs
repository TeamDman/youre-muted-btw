#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::eyre::eyre!($msg).into());
    };
    ($err:expr $(,)?) => {
        return Err($crate::eyre::eyre!($err).into());
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::eyre::eyre!($fmt, $($arg)*).into());
    };
}

#[macro_export]
macro_rules! air {
    ($msg:literal $(,)?) => ({
        let error = $crate::eyre::eyre!($msg);
        let error = $crate::error::WindyError::from(error);
        error
    });
    ($err:expr $(,)?) => ({
        let error = $crate::eyre::eyre!($err);
        let error = $crate::error::WindyError::from(error);
        error
    });
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::WindyError::from($crate::eyre::eyre!($fmt, $($arg)*))
    };
}
