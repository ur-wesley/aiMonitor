#![windows_subsystem = "windows"]

#[cfg(windows)]
fn attach_parent_console() {
    use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
const fn attach_parent_console() {}

#[cfg(windows)]
fn detach_console() {
    use windows::Win32::System::Console::FreeConsole;

    unsafe {
        let _ = FreeConsole();
    }
}

#[cfg(not(windows))]
const fn detach_console() {}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1].eq_ignore_ascii_case("export") {
        attach_parent_console();
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        if let Err(error) = rt.block_on(aimonitor_lib::export::run_export()) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }

    detach_console();
    aimonitor_lib::run();
}
