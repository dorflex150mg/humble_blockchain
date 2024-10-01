pub mod receiver {

    use tokio::sync::mpsc::{
        self,
        error::TryRecvError,
    };

    pub struct Receiver {
        receiver: mpsc::Receiver<String>,
    }

    impl Receiver {

        pub fn new(receiver: mpsc::Receiver<String>) -> Self {
            Receiver {
                receiver,
            }
        }
        
        pub async fn recv(&mut self) -> Result<String, TryRecvError> {
            self.receiver.try_recv()
        }
    }
}


                

