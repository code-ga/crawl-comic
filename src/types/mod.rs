pub mod thread_message;

pub struct Worker {
    pub id: usize,
    pub join_handle: tokio::task::JoinHandle<()>,
}
