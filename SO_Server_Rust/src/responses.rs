// Archivo para definir el formato de las respuestas cuando son existosas

pub fn http_response_200(body : &str) -> String {
    let json = format!("{{\"status\":200,\"message\":\"{}\"}}", body);
    format!(
        "HTTP/1.0 202 Accepted\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        json.len(),
        json
    )
}

#[cfg(test)]
mod test {
    use super::http_response_200;

    #[test]
    fn test_http_response_200() {
        let msg = "Tarea existosa";
        let response = http_response_200(msg);

        assert!(response.contains("202 Accepted"));
        assert!(response.contains("Content-Type: application/json"));
        assert!(response.contains("\"status\":200"));
        assert!(response.contains(msg));
    }
}