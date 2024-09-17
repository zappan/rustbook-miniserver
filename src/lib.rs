use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// struct Job;
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
  id: usize,
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
    let thread = thread::spawn(move || loop {
      // Here, we first call lock on the receiver to acquire the mutex, and then we call unwrap to
      // panic on any errors. Acquiring a lock might fail if the mutex is in a poisoned state, which
      // can happen if some other thread panicked while holding the lock rather than releasing the lock.
      // In this situation, calling unwrap to have this thread panic is the correct action to take.
      // Feel free to change this unwrap to an expect with an error message that is meaningful to you.
      // ---
      // The call to recv blocks, so if there is no job yet, the current thread will wait until a job becomes available.
      // let job = receiver.lock().unwrap().recv().unwrap();
      let message = receiver.lock().unwrap().recv();
      match message {
        Ok(job) => {
          println!("Worker {id} got a job. Executing...");
          job();
        }
        Err(_) => {
          println!("Worker {id} disconnected. Shutting down...");
          break; // exiting the thread::spawn()'s closure infinite loop
        }
      }
    });

    Worker {
      id,
      thread: Some(thread),
    }
  }
}

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: Option<mpsc::Sender<Job>>,
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    // on drop(), first, we’ll explicitly drop the sender before waiting for the threads to finish.
    drop(self.sender.take());

    // Dropping sender closes the channel, which indicates no more messages will be sent. When that
    // happens, all the calls to recv that the workers do in the infinite loop will return an error

    for worker in &mut self.workers {
      println!("Shutting down worker {}", worker.id);
      // worker.thread.join().unwrap();
      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

impl ThreadPool {
  /// Create a new ThreadPool.
  ///
  /// The size is the number of threads in the pool.
  ///
  /// # Panics
  ///
  /// The `new` function will panic if the size is zero.
  pub fn new(pool_size: usize) -> ThreadPool {
    assert!(pool_size > 0);

    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));

    // We want to create the threads and have them wait for code that we’ll send later.
    // The standard library’s implementation of threads doesn’t include any way to do that;
    // we have to implement it manually. We'll create a data structure called `Worker` which
    // picks up code that needs to be run and runs the code in the Worker’s thread
    let mut workers = Vec::with_capacity(pool_size);

    // create some workers and store them in the vector
    for id in 0..pool_size {
      workers.push(Worker::new(id, Arc::clone(&receiver)));
    }

    ThreadPool {
      workers,
      sender: Some(sender),
    }
  }

  pub fn execute<F>(&self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f); // puts the function on the heap

    // self.sender.send(job).unwrap();
    self.sender.as_ref().unwrap().send(job).unwrap(); // We’re calling unwrap on send for the case that sending fails.
                                                      // This might happen if, for example, we stop all our threads from
                                                      // executing, meaning the receiving end has stopped receiving new messages
  }
}
