fn main() {
    if let Err(err) = acli::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
