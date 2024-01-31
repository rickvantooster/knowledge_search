use std::{sync::{mpsc, Arc, Mutex}, thread};

pub type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Task>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender)
        }

    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let task = Box::new(f);
        self.sender.as_ref().unwrap().send(task).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            tracing::info!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Task>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message  = receiver.lock().unwrap().recv();
            match message {
                Ok(task) => {
                    task();
                },
                Err(e) => {
                    tracing::error!("Worker id {id} has encountered the following error: {e} \n Shutting down...");
                },
            }

        });
        Worker { id, thread: Some(thread) }

    }

}
