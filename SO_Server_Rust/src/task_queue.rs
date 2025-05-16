use std::{sync::{mpsc::Receiver, Arc, Mutex}, thread};
use std::sync::mpsc::Sender as MpscSender;

use crate::endpoints::{create_file, delete_file, fibonacci, generate_random_numbers, rerverse_text, sha256_hash, timestamp_iso, to_uppercase};

#[derive(Debug, Clone)]
pub enum TaskType {
    Reverse(String),
    Toupper(String),
    Sha256(String),
    Fibonacci(u64),
    Sleep(u64),
    TimeStamp,
    Random {count : usize, min : i32, max : i32},
    CreateFile {name : String, content : String},
    DeleteFile(String),
    Simulate {delay: u64, inner: Box<TaskType>},
    Help
}

#[derive(Debug)]
pub struct Task {
    pub description : String,
    pub task_type : TaskType,
    pub response_tx: MpscSender<String>,
}

#[derive(Clone)]
pub struct  WorkerStatus {
    pub id: usize, 
    pub busy: bool,
    pub description: String
}

pub fn start_workers(rx: Arc<Mutex<Receiver<Task>>>, states : Arc<Mutex<Vec<WorkerStatus>>>){
    for id in 0..4 {
        let rx_clone = rx.clone();
        let states_clone = states.clone();    

        thread::spawn(move || {
            loop {
                let task = {
                    let lock = rx_clone.lock().unwrap();
                    lock.recv().ok()
                };

                if let Some(task) = task {
                    {
                        let mut state = states_clone.lock().unwrap();
                        if let Some(worker) = state.iter_mut().find(|w| w.id == id) {
                            worker.busy = true;
                            worker.description = task.description.clone();
                        }
                    }

                    process_task(task);

                    {
                        let mut state = states_clone.lock().unwrap();
                        if let Some(worker) = state.iter_mut().find(|w| w.id == id) {
                            worker.busy = false;
                            worker.description = "idle".to_string();
                        }
                    }
                }
            }
        });

        states.lock().unwrap().push(WorkerStatus { id: id, busy: false, description: "idle".to_string() });
    }
}

pub fn process_task(task : Task) {
    match task.task_type {
        TaskType::Reverse(ref s) => {
            let reversed: String = rerverse_text(s);
            let _ = task.response_tx.send(reversed);
        }TaskType::Toupper(ref s) => {
            let upper = to_uppercase(s);
            let _ = task.response_tx.send(upper);
        }TaskType::Sha256(ref s) => {
            let hash = sha256_hash(s);
            let _ = task.response_tx.send(hash);
        }TaskType::Fibonacci(n) => {
            let result = fibonacci(n);
            let _ = task.response_tx.send(result.to_string());
        }TaskType::Sleep(n) => {
            std::thread::sleep(std::time::Duration::from_secs(n));
            let _ = task.response_tx.send(format!("Simulado por {} segundos", n));
        }TaskType::TimeStamp => {
            let iso = timestamp_iso();
            let _ = task.response_tx.send(iso);
        }TaskType::Random { count, min, max } => {
            let values = generate_random_numbers(count, min, max);
            let str = format!("{:?}", values);
            let _ = task.response_tx.send(str);
        }TaskType::CreateFile { ref name, ref content } => {
            let result = create_file(name, content);
            let str = match result {
                Ok(msg) => format!("{}", msg),
                Err(msg) => format!("{}", msg)
            };
            let _ = task.response_tx.send(str);
        }TaskType::DeleteFile(ref name) => {
            let result = delete_file(name);
            let str = match result {
                Ok(msg) => format!("{}", msg),
                Err(msg) => format!("{}", msg)
            };
            let _ = task.response_tx.send(str);
        }TaskType::Simulate { delay, inner } => {
            std::thread::sleep(std::time::Duration::from_secs(delay));

            // Se va a clonar la tarea que viene dentro del simulate
            let new_task = Task {
                description : task.description,
                task_type : *inner,
                response_tx : task.response_tx
            };

            process_task(new_task);
        } TaskType::Help => {
            let ayuda = format!("\"endpoints\" : [
            {{\"path\" : \"reverse\", 
            \"description\" : \"Invierte el texto recibido\", 
            \"params\" : [\"text: texto que se desea invertir\"], 
            \"example\" : \"/reverse?text=abc\"}},
            {{\"path\" : \"toupper\", \"description\" : \"Convierte el texto a mayúsculas\", \"params\" : [\"text: texto a convertir\"], \"example\" : \"/toupper?text=hola\"}},
            {{\"path\" : \"sha256\", \"description\" : \"Devuelve el hash SHA-256 del texto\", \"params\" : [\"text: texto a hashear\"], \"example\" : \"/sha256?text=hola\"}},
            {{\"path\" : \"fibonacci\", \"description\" : \"Calcula el n-ésimo número de Fibonacci (recursivo)\", \"params\" : [\"num: número a calcular\"], \"example\" : \"/fibonacci?num=10\"}},
            {{\"path\" : \"random\", \"description\" : \"Genera una lista de números aleatorios\", \"params\" : [\"count: cantidad\", \"min: mínimo\", \"max: máximo\"], \"example\" : \"/random?count=5&min=10&max=100\"}},
            {{\"path\" : \"timestamp\", \"description\" : \"Devuelve la hora actual en formato ISO\", \"params\" : [], \"example\" : \"/timestamp\"}},
            {{\"path\" : \"sleep\", \"description\" : \"Simula una espera bloqueante de N segundos\", \"params\" : [\"seconds: segundos a esperar\"], \"example\" : \"/sleep?seconds=3\"}},
            {{\"path\" : \"createfile\", \"description\" : \"Crea un archivo con el contenido indicado\", \"params\" : [\"name: nombre del archivo\", \"content: contenido\"], \"example\" : \"/createfile?name=miarchivo&content=hola\"}},
            {{\"path\" : \"deletefile\", \"description\" : \"Elimina un archivo existente\", \"params\" : [\"name: nombre del archivo\"], \"example\" : \"/deletefile?name=miarchivo\"}},
            {{\"path\" : \"simulate\", \"description\" : \"Simula un endpoint como reverse, toupper, etc., con retardo\", \"params\" : [\"seconds: retardo\", \"task: nombre del endpoint interno\", \"otros: según la tarea\"], \"example\" : \"/simulate?seconds=2&task=reverse&text=hola\"}},
            {{\"path\" : \"loadtest\", \"description\" : \"Encola múltiples tareas para medir carga del sistema\", \"params\" : [\"task: tipo de tarea\", \"count: cuántas tareas\", \"text: valor base si aplica\"], \"example\" : \"/loadtest?task=reverse&count=5&text=hola\"}},
            {{\"path\" : \"help\", \"description\" : \"Devuelve este manual de uso de endpoints\", \"params\" : [], \"example\" : \"/help\"}},
            ]");
            let _ = task.response_tx.send(ayuda);
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;
    use std::{sync::mpsc::channel};
    use super::*;

    use super::{process_task, TaskType};

    #[test]
    fn test_reverser_task() {
        let (tx, rx) = channel();
        let task = Task {
            description : "Invertir cadena".into(),
            task_type : TaskType::Reverse("abc".into()),
            response_tx : tx,
        };
        process_task(task);
        let result = rx.recv().unwrap();
        assert_eq!(result, "cba");
    }

    #[test]
    fn test_toupper_task() {
        let (tx, rx) = channel();
        let task = Task {
            description : "Mayusculas".into(),
            task_type : TaskType::Toupper("hola".into()),
            response_tx : tx,
        };

        process_task(task);
        let result = rx.recv().unwrap();
        assert_eq!(result, "HOLA");
    }

    #[test]
    fn test_fibonacci_task() {
        let (tx, rx) = channel();
        let task = Task {
            description: "Fibonacci de 6".into(),
            task_type : TaskType::Fibonacci(6),
            response_tx : tx
        };

        process_task(task);
        let result = rx.recv().unwrap();
        assert_eq!(result, "8");
    }

    #[test]
    fn test_random_task() {
        let (tx, rx) = channel();
        let task = Task {
            description: "Random".into(),
            task_type: TaskType::Random { count: 3, min: 1, max: 10 },
            response_tx: tx,
        };

        process_task(task);
        let result = rx.recv().unwrap();
        assert!(result.contains("[") && result.contains("]"));
    }
    
    #[test]
    fn test_timestamp_task() {
        let (tx, rx) = channel();
        let task = Task {
            description: "timestamp".into(),
            task_type: TaskType::TimeStamp,
            response_tx: tx
        };

        process_task(task);
        let result = rx.recv().unwrap();
        assert!(result.contains("T"));
    }

    #[test]
    fn test_simulate_reverse_task() {
        let (tx, rx) = channel();
        let task = Task {
            description: "Simular Reverse".into(),
            task_type: TaskType::Simulate { delay: 1, inner: Box::new(TaskType::Reverse("xyz".into())) },
            response_tx: tx
        };

        let start = Instant::now();
        process_task(task);
        let elapsed = start.elapsed().as_secs();

        let result = rx.recv().unwrap();
        assert_eq!(result, "zyx");
        assert!(elapsed >= 1);
    }

    #[test]
    fn test_help_task() {
        let (tx, rx) = channel();
        let task = Task {
            description: "Ayuda".into(),
            task_type: TaskType::Help,
            response_tx: tx
        };

        process_task(task);
        let result = rx.recv().unwrap();
        assert!(result.contains("\"path\" : \"reverse\""));
        assert!(result.contains("\"path\" : \"loadtest\""));
    }

    #[test]
    fn test_sleep_task() {
        let (tx, rx) = channel();
        let start = Instant::now();

        let task = Task {
            description: "Simular espera".into(),
            task_type: TaskType::Sleep(1),
            response_tx: tx,
        };

        process_task(task);
        let result = rx.recv().unwrap();

        let elapsed = start.elapsed().as_secs();
        assert!(elapsed >= 1, "el tiempo de espera fue menor a 1s");
        assert!(result.contains("Simulado por 1 segundos"));
    }

    #[test]
    fn test_create_file_task() {
        let (tx, rx) = channel();
        let task = Task {
            description: "Crear archivo".into(),
            task_type: TaskType::CreateFile { name: "test_file".into(), content: "contenido de prueba".into() },
            response_tx: tx
        };

        process_task(task);
        let result = rx.recv().unwrap();
        assert!(result.contains("creado exitosamente") || result.contains("ya existe"));
    }

    #[test]
    fn test_delete_file_task() {
        let (tx1, rx1) = channel();

        //Creamos el archivo para luego eliminarlo
        let create_task = Task {
            description: "Crear archivo".into(),
            task_type: TaskType::CreateFile { name: "test_file".into(), content: "contenido temporal".into() },
            response_tx: tx1
        };

        process_task(create_task);
        let _ = rx1.recv().unwrap();

        //En este caso vamos a eliminar el archivo
        let (tx2, rx2) = channel();
        let delete_task = Task {
            description: "Eliminar archivo".into(),
            task_type: TaskType::DeleteFile("test_file".into()),
            response_tx: tx2
        };

        process_task(delete_task);
        let result = rx2.recv().unwrap();
        assert!(result.contains("eliminado exitosamente") || result.contains("no existe"));
    }

    #[test]
    fn test_sha256_task() {
        let (tx, rx) = channel();

        let input_text = "hola";
        let expected_hash = sha256_hash(input_text);
        let task = Task {
            description: "Hashear texto".into(),
            task_type: TaskType::Sha256(input_text.to_string()),
            response_tx: tx,
        };

        process_task(task);
        let result = rx.recv().unwrap();
        assert_eq!(result, expected_hash);
        assert_eq!(result.len(), 64);
    }
}