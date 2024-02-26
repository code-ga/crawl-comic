#[derive(Debug, Clone)]
pub enum ThreadMessage {
    Start(String),
    Done(Vec<String>, String, bool),
    Stop(usize),
    Retry(String),
}
