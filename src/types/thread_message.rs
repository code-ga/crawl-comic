#[derive(Debug, Clone)]
pub enum ThreadMessage {
    Start(String, i32),
    #[allow(dead_code)]
    Done(Vec<String>, String, bool),
    Stop(usize),
    Retry(String, i32),
    Exited(usize),
}
