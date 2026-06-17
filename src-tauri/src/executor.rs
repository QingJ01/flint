use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use crate::error::Result;

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Line(String),          // stdout 或 stderr 的一行
    Exit(i32),             // 退出码
}

/// 以当前用户权限运行命令，按行推送 stdout/stderr。
/// `on_cancel` 为 None 表示不可取消（Slice 0 足够）。
pub async fn run(
    argv: &[String],
    _on_cancel: Option<()>,
) -> Result<mpsc::Receiver<StreamEvent>> {
    let (tx, rx) = mpsc::channel::<StreamEvent>(64);
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped())
       .kill_on_drop(true);

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    tokio::spawn(async move {
        let mut out = BufReader::new(stdout).lines();
        let mut err = BufReader::new(stderr).lines();
        loop {
            tokio::select! {
                Ok(Some(line)) = out.next_line() => {
                    if tx.send(StreamEvent::Line(format!("[out] {line}"))).await.is_err() { break; }
                }
                Ok(Some(line)) = err.next_line() => {
                    if tx.send(StreamEvent::Line(format!("[err] {line}"))).await.is_err() { break; }
                }
                else => break,
            }
        }
        let status = child.wait().await;
        let code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
        let _ = tx.send(StreamEvent::Exit(code)).await;
    });

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn streams_echo_output() {
        let mut rx = run(&["cmd".to_string(), "/C".to_string(), "echo hello".to_string()], None).await.unwrap();
        let mut got = String::new();
        while let Some(ev) = rx.recv().await {
            if let StreamEvent::Line(l) = ev { got.push_str(&l); }
        }
        assert!(got.contains("hello"));
    }
}
