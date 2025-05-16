use std::io::Write;
use std::os::unix::process;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::{io::Read, net::TcpStream};
use std::collections::HashMap;
use std::time::Instant;

use crate::error_responses::{http_resonse_400, http_resonse_404, http_response_500_json};
use crate::responses::{http_response_200};
use crate::task_queue::{Task, TaskType, WorkerStatus};
use crate::CONNECTION_COUNT;
// Archivo para la lògica de manejo de las conexiones

pub fn handle_connection(mut stream: TcpStream, task_sender: Sender<Task>, start_time : Instant, worker_states : Arc<Mutex<Vec<WorkerStatus>>>) {
    let mut buffer = [0u8; 1024];
    let read_bytes = stream.read(&mut buffer).unwrap_or(0);

    if read_bytes == 0 {
        return;
    }

    let request = String::from_utf8_lossy(&buffer[..]);
    let (method, path) = parse_request(&request);

    println!("Solicitud: {} {}", method, path);
    let response = route_request(&path, &task_sender, start_time, worker_states);

    println!("{}", response);
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn parse_request(request: &str) -> (&str, String) {
    let mut lines = request.lines();
    if let Some(request_line) = lines.next() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return (parts[0], parts[1].to_string());
        }
    }
    ("", "".to_string())
}

pub fn route_request(path: &str, sender: &Sender<Task>, start_time : Instant, worker_states : Arc<Mutex<Vec<WorkerStatus>>>) -> String {
    let (route, params) = parse_query(path);

    match route.as_str() {
        "/fibonacci" => {
            if let Some(n_str) = params.get("num") {
                match n_str.parse::<u64>() {
                    Ok(n) => {
                        return enqueue_and_reply(sender, TaskType::Fibonacci(n), &format!("Fibonacci para {}", n))
                    }
                    Err(e) => return http_resonse_400(&e.to_string()),
                }
            } else {
                return http_resonse_400("Falta el paràmetro 'num'");
            }
        }

        "/reverse" => {
            if let Some(text) = params.get("text") {
               return  enqueue_and_reply(sender, TaskType::Reverse(text.clone()), &format!("Reverse de {}", text));
            } else {
                return http_resonse_400("Falta el paràmetro 'text'");
            }
        }

        "/toupper" => {
            if let Some(text) = params.get("text") {
                return enqueue_and_reply(sender, TaskType::Toupper(text.clone()), &format!("Touper de {}", text));
            } else {
                return http_resonse_400("Falta el paràmetro 'text'");
            }
        }

        "/hash" => {
            if let Some(text) = params.get("text") {
                return enqueue_and_reply(sender,TaskType::Sha256(text.clone()), &format!("Sha256_hash de {}", text));
            } else {
                return http_resonse_400("Falta el paràmetro 'text'");
            }
        }

        "/sleep" => {
            if let Some(sec) = params.get("seconds") {
                match sec.parse::<u64>() {
                    Ok(seconds) => {
                        return enqueue_and_reply(sender, TaskType::Sleep(seconds), &format!("Simulaciòn por {} segundos", sec));
                    }
                    Err(_) => {
                        return http_resonse_400("El parametro 'seconds' debe de ser un nùmero positivo");
                    }
                }
            }
            return http_resonse_400("Falta el paràmetro 'seconds'");
        }

        "/timestamp" => {
            return enqueue_and_reply(sender, TaskType::TimeStamp, &format!("TimeStamp actual en formato Iso"));
        }

        "/random" => {
            let count_str = params.get("count");
            let min_str = params.get("min");
            let max_str = params.get("max");

            if count_str.is_none() || min_str.is_none() || max_str.is_none() {
                return http_resonse_400("Falta alguno o varios de los 3 paràmetros que se necesitas -> 'count', 'min', 'max'")
            }

            let count = match count_str.unwrap().parse::<usize>() {
                Ok(c) => c,
                Err(_) => return http_resonse_400("El paràmetro 'count' debe de ser entero positivo"),
            };

            let min = match min_str.unwrap().parse::<i32>() {
                Ok(mi) => mi,
                Err(_) => return http_resonse_400("El paràmetro 'min' debe de ser entero positivo"),
            };

            let max = match max_str.unwrap().parse::<i32>() {
                Ok(ma) => ma,
                Err(_) => return http_resonse_400("El paràmetro 'max' debe de ser entero positivo"),
            };

            if min >= max {
                return http_resonse_400("El paràmetro 'min' debe ser menor estrico que el paràmetro 'max'")
            }

            return enqueue_and_reply(sender, TaskType::Random { count, min, max }, "Generar números aleatorios");
        }

        "/createfile" => {
            if let (Some(name), Some(content)) = (params.get("name"), params.get("content")) {
                return enqueue_and_reply(sender, TaskType::CreateFile { name: name.clone(), content: content.clone() }, &format!("Crear archivo '{}'", name));
            }
            return http_resonse_400("Los paràmetros 'name' y 'content' son obligatorios")
        }

        "/deletefile" => {
            if let Some(name) = params.get("name") {
                return enqueue_and_reply(sender, TaskType::DeleteFile(name.clone()), &format!("Eliminar archivo '{}'", name));
            }
            return http_resonse_400("El paràmetro 'name' es obligatorio");
        }

        "/simulate" => {
            let delay = match params.get("seconds") {
                Some(s) => match s.parse::<u64>() {
                    Ok(val) => val,
                    Err(_) => return http_resonse_400("EL parametro 'seconds' debe entero positivo")
                },
                None => return http_resonse_400("Falta el parametro 'seconds'"),
            };

            let task = match params.get("task") {
                Some(t) => t.as_str(),
                None => return http_resonse_400("Falta el parametro 'task'"),
            };

            match task {
                "reverse" => {
                    if let Some(text) = params.get("text") {
                        let inner = TaskType::Reverse(text.clone());
                        return enqueue_and_reply(sender, TaskType::Simulate { delay: delay, inner: Box::new(inner) }, "Simular reverse");
                     } else {
                        return http_resonse_400("Falta el parametro 'text'")
                     }
                }

                "toupper" => {
                    if let Some(text) = params.get("text") {
                        let inner = TaskType::Toupper(text.clone());
                        return enqueue_and_reply(sender, TaskType::Simulate { delay: delay, inner: Box::new(inner) }, "Simulate toupper");
                    } else {
                        return http_resonse_400("Falta el parametro 'text'")
                    }
                }

                "fibonacci" => {
                    if let Some(n_str) = params.get("num") {
                        match n_str.parse::<u64>() {
                            Ok(n) => {
                                let inner = TaskType::Fibonacci(n);
                                return enqueue_and_reply(sender, TaskType::Simulate { delay: delay, inner: Box::new(inner) }, "Simulate fibonacci");
                            }
                            Err(_e) => return http_resonse_400("EL parametro 'num' debe de ser entero positivo"),
                        }
                    } else {
                        return http_resonse_400("Falta el parametro 'num'")
                    }
                }

                "hash" => {
                    if let Some(text) = params.get("text") {
                        let inner = TaskType::Sha256(text.clone());
                        return enqueue_and_reply(sender, TaskType::Simulate { delay: delay, inner: Box::new(inner) }, "Simulate hash");
                    } else {
                        return http_resonse_400("Falta el parametro 'text'")
                    }
                }

                "timestamp" => {
                    let inner = TaskType::TimeStamp;
                    return enqueue_and_reply(sender, TaskType::Simulate { delay, inner: Box::new(inner) }, "Simulate timestamp");
                }
                
                "random" => {
                    let count = params.get("count").and_then(|s| s.parse::<usize>().ok());
                    let min = params.get("min").and_then(|s| s.parse::<i32>().ok());
                    let max = params.get("max").and_then(|s| s.parse::<i32>().ok());

                    match (count, min, max) {
                        (Some(c), Some(a), Some(b)) if a <= b => {
                            let inner = TaskType::Random { count: c, min: a, max: b };
                            return enqueue_and_reply(sender, TaskType::Simulate { delay: delay, inner: Box::new(inner) }, "Simulate random");
                        }
                        _ => format!("Faltan paramentros")
                    }
                }

                "createfile" => {
                    if let (Some(name), Some(content)) = (params.get("name"), params.get("content")) {
                        let inner = TaskType::CreateFile { name: name.clone(), content: content.clone() };
                        return enqueue_and_reply(sender, TaskType::Simulate { delay: delay, inner: Box::new(inner) }, "Simulate createfile");
                    }
                    return http_resonse_400("Falta alguno de los siguiente parametros: 'name' o 'content'")
                }

                "deletefile" => {
                    if let Some(name) = params.get("name") {
                        let inner = TaskType::DeleteFile(name.clone());
                        return enqueue_and_reply(sender, TaskType::Simulate { delay: delay, inner: Box::new(inner) }, "Simulate deletefile");
                    }
                    return http_resonse_400("Falta el parametro 'name'")
                }
                _ => return http_resonse_400("Tarea no soportada por simulate")
            }
        }

        "/loadtest" => {
            let count = params.get("count").and_then(|v| v.parse::<usize>().ok()).unwrap_or(10);
            let task_name = params.get("task").map(|s| s.as_str()).unwrap_or("default");
            let text = params.get("text").cloned().unwrap_or_else(|| "default".to_string());

            let task_type_template = match task_name {
                "reverse" => TaskType::Reverse(text),
                "toupper" => TaskType::Toupper(text),
                "sha256" => TaskType::Sha256(text),
                "timestamp" => TaskType::TimeStamp,
                _ => return http_resonse_400("Tarea no soportada para loadtest"),
            };

            let start = Instant::now();
            let mut receivers = Vec::with_capacity(count);

            //Vamos a encolar las tareas
            for _ in 0..count {
                let (tx, rx) = mpsc::channel::<String>();
                let task = Task {
                    description : format!("Loadtest para {}", task_name),
                    task_type : task_type_template.clone(),
                    response_tx : tx,
                };
                if let Err(_) = sender.send(task) {
                    return http_response_500_json("Fallo al encolar tarea");
                }
                receivers.push(rx);
            }

            //Aquì en esta secciòn esperamos todas las respuestas
            let mut results = Vec::new();
            for rx in receivers {
                if let Ok(res) = rx.recv() {
                    results.push(res);
                } else {
                    results.push("ERROR en respuesta".to_string());
                }
            }

            let elapsed = start.elapsed();
            let sample = results.iter().take(5).cloned().collect::<Vec<_>>();

            let json = format!("{{\"task\":\"{}\",\"total_tasks\":\"{}\", \"duration_ms\" : {}, \"results_sample\" : {:?}}}", task_name, count, elapsed.as_millis(), sample);

            return http_response_200(&json);
        }
        
        "/help" => {
            return enqueue_and_reply(sender, TaskType::Help, &format!("Manual para usar los endpoints"));
        }

        "/status" => {
            let uptime = start_time.elapsed().as_secs();

            let worker_states_clone = match worker_states.lock() {
                Ok(ws) => ws.clone(),
                Err(_) => return http_response_500_json("No se pudo accerder a los datos de los workers"),
            };

            let workers_json : Vec<String> = worker_states_clone.iter().map(|w| {
                format!("{{\"id\" : {}, \"status\" : \"{}\", \"description\" : \"{}\"}}", w.id, if w.busy {"ocupado"} else {"disponible"}, w.description)
            }).collect();

            let response = format!("{{\"status\" : 200, \"pid\" : {}, \"uptime_secs\": {}, \"conexiones\": {}, \"workers\" : [{}]}}", process::parent_id(), uptime, CONNECTION_COUNT.load(Ordering::SeqCst), workers_json.join(","));

            return http_response_200(&response);
        }
        _ => http_resonse_404("Ruta no encontrada")
    }
}

fn parse_query (path: &str) -> (String, HashMap<String, String>) {
    let mut parts = path.split('?');
    let route = parts.next().unwrap_or("").to_string();
    let mut query_map = HashMap::new();

    if let Some(query) = parts.next() {
        for param in query.split('&') {
            let mut kv = param.split('=');
            let key = kv.next().unwrap_or("").to_string();
            let value = kv.next().unwrap_or("").to_string();
            query_map.insert(key, value);
        }
    }

    (route, query_map)
}

pub fn enqueue_and_reply(sender: &Sender<Task>, task_type: TaskType, desc: &str) -> String {
    let (response_tx, response_rx) = mpsc::channel::<String>();

    let task = Task {
        description: desc.to_string(),
        task_type,
        response_tx,
    };

    if let Err(_) = sender.send(task) {
        return http_response_500_json("No se pudo encolar la tarea");
    }

    // Esperamos la respuesta del worker (bloqueante)
    match response_rx.recv() {
        Ok(json_result) => http_response_200(&json_result),
        Err(_) => http_response_500_json("Error al recibir resultado de la tarea"),
    }
}


#[cfg(test)]
mod test {
    use std::{sync::mpsc::channel, thread};

    use crate::task_queue::{Task, TaskType};

    use super::enqueue_and_reply;


    #[test]
    fn test_enqueue_and_reply_success() {
        let (reply_tx, reply_rx) = channel::<Task>();

        //Simula un worker trabajando
        thread::spawn(move || {
            if let Ok(task) = reply_rx.recv() {
                let _ = task.response_tx.send("resultado_ok".to_string());
            }
        });

        let response = enqueue_and_reply(&reply_tx, TaskType::Reverse("abc".into()), "Reverse text");
        assert!(response.contains("HTTP/1.0 202 Accepted"));
        assert!(response.contains("resultado_ok"));
    }

    #[test]
    fn test_enqueue_and_reply_send_error() {
        let (reply_tx, reply_rx) = channel::<Task>();
        drop(reply_rx); //Aquì se cierra el receiver

        let response = enqueue_and_reply(&reply_tx, TaskType::Reverse("abc".into()), "reverse text");
        assert!(response.contains("500"));
        assert!(response.contains("No se pudo encolar la tarea"));
    }

    #[test]
    fn test_enqueue_and_reply_recv_error() {
        let (reply_tx, reply_rx) = channel::<Task>();

        thread::spawn(move || {
            if let Ok(task) = reply_rx.recv() {
                drop(task.response_tx);
            }
        });

        let response = enqueue_and_reply(&reply_tx, TaskType::Reverse("abc".into()), "reverse test");
        assert!(response.contains("500"));
        assert!(response.contains("Error al recibir resultado"));
    }
}