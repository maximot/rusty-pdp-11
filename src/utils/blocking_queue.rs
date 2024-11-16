use std::sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex};

pub struct BlockingQueue<T> {
    sender: Sender<T>,
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> BlockingQueue<T> {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender: sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn push(&self, e: T) {
        self.sender.send(e).ok();
    }

    pub fn pop(&self) -> Option<T> {
        self.receiver.lock().unwrap().try_recv().ok()
    }

    pub fn pop_blocking(&self) -> Option<T> {
        self.receiver.lock().unwrap().recv().ok()
    }
}

impl<T> Clone for BlockingQueue<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}