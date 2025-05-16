#[cfg(test)]
mod test {
    use std::{thread, sync::{mpsc::channel, Arc, Mutex}, time::Duration};
    use so_server_rust::task_queue::{start_workers, Task};

    #[test]
    fn test_start_workers_executes_task_and_updates_status() {
        let (tx, rx) = channel();
        let new_rx = Arc::new(Mutex::new(rx));
        let states = Arc::new(Mutex::new(Vec::new()));

        //Lanzar a los workers
        start_workers(new_rx.clone(), states.clone());

        // Se encola una tarea
        let (resp_tx, resp_rx) = channel();
        let task = Task {
            description: "Test de reverse".into(),
            task_type: so_server_rust::task_queue::TaskType::Reverse("abc".to_string()),
            response_tx: resp_tx
        };
        tx.send(task).unwrap();

        let result = resp_rx.recv_timeout(Duration::from_secs(1)).expect("No se recibio ninguna respuesta");
        assert_eq!(result, "cba");

        thread::sleep(Duration::from_millis(100));

        let status = states.lock().unwrap();
        assert_eq!(status.len(), 4);

        let idle_workers: Vec<_> = status.iter().filter(|w| w.description == "idle").collect();
        assert!(!idle_workers.is_empty(), "No hay workers marcados como 'idle'");
    }
}
