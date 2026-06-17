use tokio::io::{AsyncRead, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use crate::error::{FlintError, Result};

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Line(String),          // stdout 或 stderr 的一行
    Exit(i32),             // 退出码
}

async fn pump<R: AsyncRead + Unpin>(stream: R, tx: mpsc::Sender<StreamEvent>, tag: &'static str) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if tx.send(StreamEvent::Line(format!("[{tag}] {line}"))).await.is_err() {
            return; // receiver dropped — stop pumping
        }
    }
}

/// 以当前用户权限运行命令，按行推送 stdout/stderr（两条流独立读取，互不截断）。
pub async fn run(argv: &[String], _on_cancel: Option<()>) -> Result<mpsc::Receiver<StreamEvent>> {
    if argv.is_empty() {
        return Err(FlintError::Other("empty argv".into()));
    }
    let (tx, rx) = mpsc::channel::<StreamEvent>(64);
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped())
       .kill_on_drop(true);

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");

    // 两个独立 reader 任务 —— 避免 select! 截断较慢的那条流
    let out_task = tokio::spawn(pump(stdout, tx.clone(), "out"));
    let err_task = tokio::spawn(pump(stderr, tx.clone(), "err"));

    let tx_exit = tx; // 原始 sender：两个 reader 完成后才发 Exit，再 drop 以关闭 channel
    tokio::spawn(async move {
        let _ = out_task.await;
        let _ = err_task.await;
        let code = child.wait().await.ok().and_then(|s| s.code()).unwrap_or(-1);
        let _ = tx_exit.send(StreamEvent::Exit(code)).await;
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

    #[tokio::test]
    async fn captures_both_stdout_and_stderr() {
        // 同一条命令同时写 stdout 和 stderr；修复前 select! 会截断较慢的流。
        let mut rx = run(
            &["cmd".to_string(), "/C".to_string(), "echo out & echo err 1>&2".to_string()],
            None,
        ).await.unwrap();
        let mut got = String::new();
        while let Some(ev) = rx.recv().await {
            if let StreamEvent::Line(l) = ev { got.push_str(&l); got.push('\n'); }
        }
        assert!(got.contains("[out] out"), "missing stdout line. got:\n{got}");
        assert!(got.contains("[err] err"), "missing stderr line (truncated?). got:\n{got}");
    }
}
