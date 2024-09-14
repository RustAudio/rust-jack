mod client;
mod log;
mod processing;
mod ringbuffer;
mod time;
mod transport;

#[ctor::ctor]
fn log_to_stdio() {
    crate::set_logger(crate::LoggerType::Stdio);
}
