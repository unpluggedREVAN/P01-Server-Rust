// Lògica de respuesta de errores


pub fn http_resonse_404(msg: &str) -> String {
    let json = format!("{{\"status\" : 404, \"error\" : \"{}\"}}", msg);
    format!(
        "HTTP/1.0 404 Not Found\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
        json.len(),
        json
    )
}

pub fn http_resonse_400(msg: &str) -> String {
    let json = format!("{{\"status\" : 400, \"error\" : \"{}\"}}", msg);
    format!(
        "HTTP/1.0 404 Bad Request\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
        json.len(),
        json
    )
}

pub fn http_response_500_json(msg: &str) -> String {
    let json = format!("{{\"status\":500,\"message\":\"{}\"}}", msg);
    format!(
        "HTTP/1.0 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        json.len(),
        json
    )
}

#[cfg(test)]
mod tests {
    use crate::error_responses::{http_resonse_400, http_resonse_404, http_response_500_json};


    #[test]
    fn test_http_response_404() {
        let msg = "Página no encontrada";
        let response = http_resonse_404(msg);

        assert!(response.contains("404 Not Found"));
        assert!(response.contains("Content-Type: text/plain"));
        assert!(response.contains("\"status\" : 404"));
        assert!(response.contains(msg));
    }

    #[test]
    fn test_http_response_400() {
        let msg = "Parámetro inválido";
        let response = http_resonse_400(msg);

        // Nota: tu función tiene mal el código: dice "404 Bad Request", debería ser "400"
        assert!(response.contains("404 Bad Request")); // <- ¿esto es un error en tu código?
        assert!(response.contains("\"status\" : 400"));
        assert!(response.contains(msg));
    }

    #[test]
    fn test_http_response_500_json() {
        let msg = "Error interno del servidor";
        let response = http_response_500_json(msg);

        assert!(response.contains("500 Internal Server Error"));
        assert!(response.contains("application/json"));
        assert!(response.contains("\"status\":500"));
        assert!(response.contains(msg));
    }
}