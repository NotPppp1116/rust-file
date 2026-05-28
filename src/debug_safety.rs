use std::fs;

pub fn is_being_debugged() -> bool {
    let status = match fs::read_to_string("/proc/self/status") {
        Ok(s) => s,
        Err(_) => return false,
    };

    for line in status.lines() {
        if let Some(pid) = line.strip_prefix("TracerPid:") {
            return pid.trim() != "0";
        }
    }

    false
}
