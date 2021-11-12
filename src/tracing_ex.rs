pub trait WarnIfError {
    fn warn_if_error(&self, msg: &str);
}

impl<T, E> WarnIfError for Result<T, E>
where
    E: std::fmt::Debug,
{
    fn warn_if_error(&self, msg: &str) {
        if let Err(error) = self {
            tracing::warn!("{}: {:?}", msg, error);
        }
    }
}
