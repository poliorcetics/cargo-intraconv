use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, BufRead};

fn main() {
    let args = env::args_os().skip(1);

    args.for_each(|arg| {
        let path = Path::new(&arg);

        let file = match File::open(path) {
            Ok(file) => BufReader::new(file),
            Err(err) => {
                eprintln!("Failed to open file '{}': {}", path.display(), err);
                return;
            }

        };

        for (n, line) in file.lines().enumerate() {
            let line = match line {
                Ok(s) => s,
                Err(err) => {
                    eprintln!("Failed to read line {}: {}", n, err);
                    return;
                }
            };
            println!("{}", line);
        }
    });
}
