macro_rules! t {
    ($arg:expr) => (
        log::trace!("{}", $arg)
    );
    ($($arg:tt)+) => (
        log::trace!($($arg)+)
    );
}

pub(crate) use t;