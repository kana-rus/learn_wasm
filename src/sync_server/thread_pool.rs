use std::{
    thread,
    sync::{Arc, Mutex, mpsc},
};
use super::utils::types::{
    Message,
    ServerError,
};


struct Worker {
    id:     usize,
    thread: Option<  // for use take() to task ownership of each thread in ThreadPool's drop()
        thread::JoinHandle<Result<(), ServerError>>
    >,
} impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Result<Self, ServerError> {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver
                    .lock()?
                    .recv()?;
                match message {
                    Message::NewJob(job) => {
                        println!("Worker {id} got a job; executing");
                        job()
                    },
                    Message::Terminate => {
                        println!("Worker {id} was tokd to terminate");
                        break;
                    }
                }
            }
            Ok(())
        });

        Ok(Self { id, thread: Some(thread) })
    }
}

pub(super) struct ThreadPool {
    workers: Vec<Worker>,
    sender:  mpsc::Sender<Message>,
} impl ThreadPool {

    pub fn new(size: usize) -> Result<Self, ServerError> {
        if size <= 0 {
            return Err(ServerError::BadRequest("size of therad pool has to be positive".into()))
        }
        let (sender, receiver) = mpsc::channel::<Message>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(
                Worker::new(id, Arc::clone(&receiver))?
            )
        }

        Ok(Self { workers, sender })
    }

    pub fn execute(&self, f: impl FnOnce() + Send + 'static) -> Result<(), ServerError> {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job))?;
        Ok(())
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).expect("failed to send terminate")
        }
        println!("shutting down all workers...");

        for worker in &mut self.workers {
            println!("shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join()
                    .expect("failed to join thread")
                    .expect("thread failed");
            }
        }
    }
}