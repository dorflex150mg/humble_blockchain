use tokio::sync::mpsc::{self, error::TryRecvError};

/// Wraps an `mpsc::Receiver<String>`.
pub struct Receiver {
    receiver: mpsc::Receiver<String>,
}

impl Receiver {
    /// Creates a new `[Receiver]`.
    #[must_use]
    pub fn new(receiver: mpsc::Receiver<String>) -> Self {
        Receiver { receiver }
    }

    /// Read a String from the channel or fails with a `TryRecvError`.
    #[allow(clippy::unused_async)]
    pub async fn recv(&mut self) -> Result<String, TryRecvError> {
        self.receiver.try_recv()
    }
}
