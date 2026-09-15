fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1].eq_ignore_ascii_case("export") {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        if let Err(error) = rt.block_on(aimonitor_lib::export::run_export()) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }

    aimonitor_lib::run();
}
