use std::{path::Path, process::Stdio};

use tokio::{fs::File, io, process::Command, task};

pub fn assert_dir_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        Err(format!("path {} does not exist", path.to_str().unwrap()))
    } else if !path.is_dir() {
        Err(format!(
            "path {} is not a directory",
            path.to_str().unwrap()
        ))
    } else {
        Ok(())
    }
}

pub fn assert_file_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        Err(format!("path {} does not exist", path.to_str().unwrap()))
    } else if !path.is_file() {
        Err(format!("path {} is not a file", path.to_str().unwrap()))
    } else {
        Ok(())
    }
}

pub async fn run_command(
    command: &str,
    args: &[&str],
    stdout_file: Option<&Path>,
    stderr_file: Option<&Path>,
) -> Result<(), String> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn command {} {:?}: {}", command, args, e))?;
    let mut handles = vec![];
    if let Some(stdout_file) = stdout_file {
        let mut file = File::create(stdout_file)
            .await
            .map_err(|e| format!("failed to create stdout file: {}", e))?;
        let mut stdout = child.stdout.take().unwrap();
        handles.push(task::spawn(async move {
            io::copy(&mut stdout, &mut file).await.unwrap();
        }));
    }
    if let Some(stderr_file) = stderr_file {
        let mut file = File::create(stderr_file)
            .await
            .map_err(|e| format!("failed to create stdout file: {}", e))?;
        let mut stderr = child.stderr.take().unwrap();
        handles.push(task::spawn(async move {
            io::copy(&mut stderr, &mut file).await.unwrap();
        }));
    }
    let exit_status = child
        .wait()
        .await
        .map_err(|e| format!("failed when waiting on child: {}", e))?;
    let res = if exit_status.success() {
        Ok(())
    } else {
        Err(format!(
            "`{} {:?}` command returned exit status {}",
            command, args, exit_status
        ))
    };
    while let Some(handle) = handles.pop() {
        handle.await.map_err(|e| format!("task panicked: {}", e))?;
    }
    res
}

pub async fn run_command_with_output(
    command: &str,
    args: &[&str],
    stderr_file: Option<&Path>,
) -> Result<String, String> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn command {} {:?}: {}", command, args, e))?;
    let mut handles = vec![];
    if let Some(stderr_file) = stderr_file {
        let mut file = File::create(stderr_file)
            .await
            .map_err(|e| format!("failed to create stdout file: {}", e))?;
        let mut stderr = child.stderr.take().unwrap();
        handles.push(task::spawn(async move {
            io::copy(&mut stderr, &mut file).await.unwrap();
        }));
    }
    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("failed when waiting on child: {}", e))?;
    let res = if output.status.success() {
        if let Ok(s) = String::from_utf8(output.stdout.clone()) {
            Ok(s)
        } else {
            Err(format!("failed to parse stdout as utf8: {:?}", output))
        }
    } else {
        Err(format!(
            "`{} {:?}` command returned exit status {}",
            command, args, output.status
        ))
    };
    while let Some(handle) = handles.pop() {
        handle.await.map_err(|e| format!("task panicked: {}", e))?;
    }
    res
}
