//! Minimaler LSP-Client für Tinymist (JSON-RPC über stdin/stdout).
//!
//! Anfragen warten blockierend auf ihre Antwort; Benachrichtigungen des Servers
//! (z. B. `tinymist/preview/scrollSource`) gehen an einen Rückruf.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

type Pending = Arc<Mutex<HashMap<i64, Sender<Result<Value, String>>>>>;

pub struct LspClient {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    next_id: AtomicI64,
    pending: Pending,
}

impl LspClient {
    /// Startet `program lsp` und leitet Server-Benachrichtigungen an `on_notification` weiter.
    pub fn spawn(
        program: &Path,
        on_notification: impl Fn(&str, Value) + Send + 'static,
    ) -> Result<Self, String> {
        let mut child = Command::new(program)
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("{} lässt sich nicht starten: {e}", program.display()))?;
        let stdin = child.stdin.take().ok_or("kein stdin")?;
        let stdout = child.stdout.take().ok_or("kein stdout")?;
        let pending: Pending = Arc::default();

        let stdin = Arc::new(Mutex::new(stdin));
        let reader_pending = pending.clone();
        let reply_stdin = stdin.clone();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Some(message) = read_message(&mut reader) {
                match (message.get("id").and_then(Value::as_i64), message.get("method")) {
                    (Some(id), None) => {
                        if let Some(tx) = reader_pending.lock().unwrap().remove(&id) {
                            let result = match message.get("error") {
                                Some(error) => Err(error.to_string()),
                                None => Ok(message.get("result").cloned().unwrap_or(Value::Null)),
                            };
                            let _ = tx.send(result);
                        }
                    }
                    (None, Some(method)) => {
                        let params = message.get("params").cloned().unwrap_or(Value::Null);
                        on_notification(method.as_str().unwrap_or_default(), params);
                    }
                    // Anfrage des Servers an uns, z. B. `window/showDocument` beim Klick in die
                    // Vorschau: wie eine Benachrichtigung weiterreichen und beantworten.
                    (Some(id), Some(method)) => {
                        let method = method.as_str().unwrap_or_default();
                        let params = message.get("params").cloned().unwrap_or(Value::Null);
                        let result = match method {
                            "window/showDocument" => json!({ "success": true }),
                            "workspace/configuration" => json!([]),
                            _ => Value::Null,
                        };
                        on_notification(method, params);
                        let _ = write_message(&reply_stdin, &json!({ "jsonrpc": "2.0", "id": id, "result": result }));
                    }
                    _ => {}
                }
            }
        });

        Ok(Self { child, stdin, next_id: AtomicI64::new(1), pending })
    }

    pub fn request(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = channel();
        self.pending.lock().unwrap().insert(id, tx);
        self.write(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))?;
        rx.recv_timeout(Duration::from_secs(30))
            .map_err(|_| format!("Keine Antwort auf {method}"))?
    }

    pub fn notify(&self, method: &str, params: Value) -> Result<(), String> {
        self.write(&json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    fn write(&self, message: &Value) -> Result<(), String> {
        write_message(&self.stdin, message)
    }
}

fn write_message(stdin: &Mutex<ChildStdin>, message: &Value) -> Result<(), String> {
    let body = message.to_string();
    let mut stdin = stdin.lock().unwrap();
    write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len())
        .and_then(|_| stdin.flush())
        .map_err(|e| format!("Tinymist nicht erreichbar: {e}"))
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_message(reader: &mut impl BufRead) -> Option<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':')
            && key.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let mut body = vec![0; length?];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reads_framed_messages() {
        let body = r#"{"jsonrpc":"2.0","id":1,"result":null}"#;
        let raw = format!("Content-Length: {}\r\n\r\n{body}Content-Length: 2\r\n\r\n{{}}", body.len());
        let mut reader = Cursor::new(raw.into_bytes());
        assert_eq!(read_message(&mut reader).unwrap()["id"], 1);
        assert_eq!(read_message(&mut reader).unwrap(), json!({}));
        assert!(read_message(&mut reader).is_none());
    }
}
