fn main() -> std::process::ExitCode {
  match pplr::cli::run(std::env::args().skip(1).collect()) {
    Ok(code) => code,
    Err(error) => {
      eprintln!("pplr: {}", pplr::clean(&error.to_string()));
      std::process::ExitCode::FAILURE
    }
  }
}
