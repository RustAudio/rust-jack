mod client;
mod log;
mod processing;
mod time;

#[ctor::ctor]
fn log_to_stdio() {
    crate::set_logger(crate::LoggerType::Stdio);
}
