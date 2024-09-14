mod client;
mod log;
mod time;

#[ctor::ctor]
fn log_to_stdio() {
    crate::set_logger(crate::LoggerType::Stdio);
}
