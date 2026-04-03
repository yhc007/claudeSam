//! 백그라운드 데몬화
//! 
//! 터미널에서 분리되어 백그라운드에서 실행

use anyhow::Result;
use daemonize::Daemonize;
use std::fs::File;
use std::path::PathBuf;

/// 데몬 설정
pub struct DaemonizeConfig {
    pub pid_file: PathBuf,
    pub stdout: PathBuf,
    pub stderr: PathBuf,
    pub working_dir: PathBuf,
}

impl DaemonizeConfig {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            pid_file: base_path.join(".kairos.pid"),
            stdout: base_path.join("kairos.log"),
            stderr: base_path.join("kairos.err"),
            working_dir: base_path,
        }
    }
}

/// 프로세스를 데몬으로 전환
pub fn daemonize(config: &DaemonizeConfig) -> Result<()> {
    // 로그 디렉토리 생성
    std::fs::create_dir_all(&config.working_dir)?;
    
    let stdout = File::create(&config.stdout)?;
    let stderr = File::create(&config.stderr)?;

    let daemonize = Daemonize::new()
        .pid_file(&config.pid_file)
        .chown_pid_file(true)
        .working_directory(&config.working_dir)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before dropping privileges");

    daemonize.start()?;
    
    Ok(())
}

/// PID 파일에서 실행 중인 데몬 PID 읽기
pub fn read_daemon_pid(pid_file: &PathBuf) -> Option<u32> {
    std::fs::read_to_string(pid_file)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// 데몬 종료
pub fn stop_daemon(pid_file: &PathBuf) -> Result<bool> {
    if let Some(pid) = read_daemon_pid(pid_file) {
        #[cfg(unix)]
        {
            use std::process::Command;
            let output = Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .output()?;
            
            if output.status.success() {
                // PID 파일 삭제
                let _ = std::fs::remove_file(pid_file);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// 데몬 실행 중인지 확인
pub fn is_daemon_running(pid_file: &PathBuf) -> bool {
    if let Some(pid) = read_daemon_pid(pid_file) {
        #[cfg(unix)]
        {
            use std::process::Command;
            return Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        }
    }
    false
}
