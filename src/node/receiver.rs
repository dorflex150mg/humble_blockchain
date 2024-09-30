pub mod Receiver {

    struct Receiver {
        receiver: mpsc::Receiver,
    }

    impl Receiver {
        
        pub async fn recv() -> Result<String, TryRecvError> {
            self.receiver.try_recv()
        }
    }
}


                

