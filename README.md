# P01-Server-Rust  

**Estudiantes:** Darío Espinoza Aguilar y Jose Pablo Agüero Mora  
**Curso:** Principios de Sistemas Operativos  

Servidor web concurrente y asíncrono básico en Rust, incluye:  
- Conexiones TCP  
- Enrutado y parseo de consultas HTTP  
- Cola de tareas con pool de trabajadores (*workers*)  
- Gestión de estados y métricas (conexiones, uptime, estado de workers)  

---

## Requisitos  

- [Rust y Cargo](https://rustup.rs)  
- Git (para clonar el repositorio)  
- *(Opcional)* [`cargo-watch`](https://crates.io/crates/cargo-watch) para recarga automática  

---

## Instalación y compilación  

1. **Clonar el repositorio:**  
   ```bash
   git clone https://github.com/unpluggedREVAN/P01-Server-Rust.git
   cd P01-Server-Rust
   ```

2. **(Opcional) Variables de entorno:**  
   - Crea `.env` en la raíz para personalizar:  
     ```bash
     echo "RUST_LOG=debug" > .env
     echo "PORT=7878" >> .env
     ```

3. **Compilar:**  
   ```bash
   cargo build --release
   ```

4. **Ejecutar:**  
   ```bash
   cargo run
   ```  
   *Por defecto escucha en `127.0.0.1:7878`.*  

---

## Endpoints disponibles  

> Todas las respuestas incluyen `Content-Type: application/json`.  

| Ruta           | Método | Parámetros                                                                                     | Descripción                                                             |
|----------------|--------|-----------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|
| `/health`      | GET    | —                                                                                             | Comprueba que el servidor está activo.                                  |
| `/reverse`     | GET    | `text=...`                                                                                    | Invierte el texto recibido.                                             |
| `/toupper`     | GET    | `text=...`                                                                                    | Convierte texto a MAYÚSCULAS.                                           |
| `/hash`        | GET    | `text=...`                                                                                    | Hash SHA-256 del texto.                                                 |
| `/fibonacci`   | GET    | `num=<n>`                                                                                     | Calcula el n-ésimo número Fibonacci (recursivo).                        |
| `/sleep`       | GET    | `seconds=<n>`                                                                                 | Simula retardo bloqueante de *n* segundos.                              |
| `/timestamp`   | GET    | —                                                                                             | Devuelve hora actual en formato ISO8601.                                |
| `/random`      | GET    | `count=<n>&min=<a>&max=<b>`                                                                   | Genera *n* números aleatorios entre *a* y *b*.                          |
| `/createfile`  | GET    | `name=<fname>&content=<text>`                                                                 | Crea `archivos/<fname>.txt` con contenido.                              |
| `/deletefile`  | GET    | `name=<fname>`                                                                                | Elimina el archivo `archivos/<fname>.txt`.                              |
| `/simulate`    | GET    | `seconds=<d>&task={reverse,toupper,hash,fibonacci,timestamp,random,createfile,deletefile}`<br>`&...[params de la tarea]` | Simula cualquier endpoint con retardo *d*. |
| `/loadtest`    | GET    | `task={reverse,toupper,sha256,timestamp}`<br>`&count=<n>&text=<base>`                         | Encola múltiples tareas para medir carga y devuelve estadística.        |
| `/status`      | GET    | —                                                                                             | Reporta métricas: PID, uptime, conexiones totales y estado de workers. |
| `/help`        | GET    | —                                                                                             | Manual JSON de uso de todos los endpoints.                              |

---

## Arquitectura interna  

### 1. `main.rs`  
- Inicializa contador atómico de conexiones (`CONNECTION_COUNT`).  
- Arranca un pool de *4 workers* desde `task_queue::start_workers`.  
- Escucha en TCP y, por cada conexión, lanza un hilo con `handle_connection`.  

### 2. `handle_connection.rs`  
- Lee la solicitud HTTP cruda (hasta 1024 B).  
- Extrae `method` y `path` (con query string).  
- Invoca `route_request` para encolar tareas o generar respuestas inmediatas.  
- Escribe y hace *flush* de la respuesta.  

### 3. `task_queue.rs`  
- Define `TaskType` (tipos de tarea) y `Task` con canal de respuesta.  
- `WorkerStatus` mantiene estado de cada *worker* (ID, ocupado/idle, descripción).  
- `start_workers`: crea 4 hilos que reciben tareas de un canal protegido por `Mutex`.  
- `process_task`: delega a la función correspondiente en `endpoints.rs`, y envía el resultado.  

### Módulos auxiliares  
- `endpoints.rs`: implementa la lógica de cada endpoint.  
- `responses.rs` y `error_responses.rs`: formateo uniforme de respuestas HTTP.  

---

## Pruebas automatizadas  

Ejecuta todas las pruebas unitarias y de integración:  
```bash
cargo test
```

---

## Recarga automática (opcional)  

Para desarrollo con recarga en caliente:  
```bash
cargo install cargo-watch
cargo watch -x run
```  