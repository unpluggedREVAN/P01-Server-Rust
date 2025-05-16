mod handle_connection;
mod endpoints;
mod error_responses;
mod responses;
mod task_queue;

use std::{net::TcpListener, sync::{atomic::{AtomicUsize, Ordering}, mpsc::{channel, Receiver, Sender}, Arc, Mutex}, thread, time::Instant};
use handle_connection::handle_connection;
use task_queue::{start_workers, Task, WorkerStatus};

static CONNECTION_COUNT: AtomicUsize = AtomicUsize::new(0);

fn main () {

    let start_time = Instant::now();
    let workers_states: Arc<Mutex<Vec<WorkerStatus>>> = Arc::new(Mutex::new(vec![]));

    let (tx, rx) : (Sender<Task>, Receiver<Task>) = channel();
    let shared_rx = Arc::new(Mutex::new(rx));

    start_workers(shared_rx, workers_states.clone());


    let listener = TcpListener::bind("127.0.0.1:7878").expect("Fallo al iniciar el server");
    println!("Servidor ejecutandose en http://127.0.0.1:7878");

    for stream in listener.incoming() {
        CONNECTION_COUNT.fetch_add(1, Ordering::SeqCst);
        let stream = stream.expect("Error de conexciòn");
        let tx_clone = tx.clone();
        let worker_states = workers_states.clone();

        thread::spawn(move || {
            handle_connection(stream, tx_clone, start_time, worker_states.clone());
        });
    }
}