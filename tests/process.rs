//
// Sysinfo
//
// Copyright (c) 2018 Guillaume Gomez
//

extern crate sysinfo;

use sysinfo::ProcessExt;
use sysinfo::SystemExt;

#[test]
fn test_process() {
    let mut s = sysinfo::System::new();
    assert_eq!(s.get_processes().len(), 0);
    s.refresh_processes();
    assert!(!s.get_processes().is_empty());
    #[cfg(not(windows))]
    assert!(s
        .get_processes()
        .values()
        .any(|p| !p.exe().to_str().unwrap_or("").is_empty()));
}

#[test]
fn test_process_refresh() {
    let mut s = sysinfo::System::new();
    assert_eq!(s.get_processes().len(), 0);
    s.refresh_process(sysinfo::get_current_pid().expect("failed to get current pid"));
    assert_eq!(
        s.get_process(sysinfo::get_current_pid().expect("failed to get current pid"))
            .is_some(),
        true
    );
}

#[test]
#[cfg(windows)]
fn test_get_cmd_line() {
    let p = std::process::Command::new("timeout")
        .arg("/t")
        .arg("3")
        .spawn()
        .unwrap();
    let mut s = sysinfo::System::new();
    assert!(s.get_processes().len() == 0);
    s.refresh_processes();
    assert!(s.get_processes().len() > 0);
    if let Some(process) = s.get_process(p.id() as sysinfo::Pid) {
        assert_eq!(process.cmd(), &["timeout", "/t", "3"]);
    } else {
        // We're very likely on a "linux-like" shell so let's try some unix command...
        unix_like_cmd();
    }
}

#[test]
#[cfg(not(windows))]
fn test_get_cmd_line() {
    unix_like_cmd();
}

fn unix_like_cmd() {
    let p = std::process::Command::new("sleep")
        .arg("3")
        .spawn()
        .unwrap();
    let mut s = sysinfo::System::new();
    assert!(s.get_processes().is_empty());
    s.refresh_processes();
    assert!(!s.get_processes().is_empty());
    let process = s.get_process(p.id() as sysinfo::Pid).unwrap();
    assert_eq!(process.cmd(), &["sleep", "3"]);
}


#[test]
fn cpu_usage_is_not_nan() {
    let mut system = sysinfo::System::new();
    system.refresh_processes();

    let first_pids = system
        .get_processes()
        .iter()
        .take(10)
        .map(|(&pid, _)| pid)
        .collect::<Vec<_>>();
    let mut checked = 0;

    first_pids.into_iter().for_each(|pid| {
        system.refresh_process(pid);
        if let Some(p) = system.get_process(pid) {
            assert!(!p.cpu_usage().is_nan());
            checked += 1;
        }
    });
    assert!(checked > 0);
}
