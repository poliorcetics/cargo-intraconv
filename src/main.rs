use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{self, BufReader, BufRead, Read};

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

        match handle_file(file) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        }
    });
}

fn handle_file<R: Read>(file: BufReader<R>) -> io::Result<()> {
    for (n, line) in file.lines().enumerate() {
        let line = line?;
        println!("{:5}:    \"{}\"", n, line);
    }

    Ok(())
}
