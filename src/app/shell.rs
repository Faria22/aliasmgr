use log::debug;
use std::os::fd::BorrowedFd;

pub fn determine_shell() -> String {
    todo!(
        "this function needs to determine the user's shell that will be exported to the environment"
    )
}

pub fn send_alias_deltas_to_shell(deltas: &str) {
    let fd3 = unsafe { BorrowedFd::borrow_raw(3) };
    nix::unistd::write(fd3, deltas.as_bytes()).expect("Failed to write alias deltas to shell");
    debug!("Sent alias deltas to shell: {}", deltas);
}
