fn main() {
    if let Err(err) = acli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
