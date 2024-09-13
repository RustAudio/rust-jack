mod client;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    crate::set_logger(crate::LoggerType::Stdio);
}
