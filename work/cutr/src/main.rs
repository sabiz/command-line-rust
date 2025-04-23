fn main() {
    if let Err(e) = cutr::get_args().and_then(cutr::run) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
