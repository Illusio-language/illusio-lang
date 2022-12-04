use std::fs;
/// Function used for reading files,
pub fn read_file(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(1)
        }
    }
}
