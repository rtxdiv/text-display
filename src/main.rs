use std::env;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::exit;
use std::thread;
use std::time::Duration;

const SCRIPT_NAME: &str = "script";

struct Config {
    dir: PathBuf,
    cycle: bool,
}

fn process_args() -> Config {
    let mut relative_path = PathBuf::new();
    let mut is_cycle: bool = false;

    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        exit(1);
    }
    let mut args_iterator = args.into_iter();
    while let Some(arg) = args_iterator.next() {
        match arg.as_str() {
            "-d" | "--dir" => {
                if let Some(val) = args_iterator.next() {
                    match fs::canonicalize(PathBuf::from(&val)) {
                        Ok(path) => {
                            if path.is_dir() {
                                relative_path = path
                            } else {
                                eprintln!("{val} is not directory");
                                exit(1);
                            }
                        },
                        Err(e) => {
                            eprintln!("{e}: {val}");
                            exit(1);
                        }
                    }
                } else {
                    print_usage();
                    exit(1);
                }
            }
            "-c" | "--cycle" => { is_cycle = true }
            unknown => {
                eprintln!("Unknown arg {unknown}");
                print_usage();
                exit(1);
            }
        }
    }
    Config { dir: relative_path, cycle: is_cycle }
}

fn print_usage() {
    println!("Usage:\ntext-display -d <dir>\ntext-display --dir <dir>");
}

fn parse_script(dir: &Path) -> Vec<(String, f32)> {
    let script_path = dir.join(SCRIPT_NAME);
    if fs::exists(&script_path).is_err() {
        eprintln!("File {SCRIPT_NAME:?} is not found in source directory");
        exit(1);
    }
    let mut script_values: Vec<(String, f32)> = Vec::new();

    match fs::read_to_string(&script_path) {
        Ok(content) => {
            for (line_num, line) in content.lines().enumerate() {
                if line.is_empty() || line.starts_with("#") { continue }
                let line_parts: Vec<&str> = line.split_ascii_whitespace().collect();
                if line_parts.len() != 2 {
                    eprintln!("Script parsing error, line {}: >>{}<<", line_num, line);
                    exit(1);
                }
                let name = line_parts[0];
                let time_s = match line_parts[1].parse::<f32>() {
                    Ok(num) => num,
                    Err(_) => {
                        eprintln!("Script parsing error, line {}: >>{}<<", line_num, line_parts[1]);
                        exit(1);
                    }
                };
                script_values.push((name.into(), time_s));
            }
        }
        Err(e) => {
            eprintln!("Unable to read script: {e}");
            exit(1);
        }
    }
    script_values
}

fn process_script(dir: &Path, script_values: &Vec<(String, f32)>) {
    for (name, seconds) in script_values {
        let file_path = dir.join(name);
        if fs::exists(&file_path).is_err() {
            print!("\x1B[2J\x1B[1;1H");
            eprintln!("File {file_path:?} is not found");
            exit(1);
        }
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                print!("\x1B[2J\x1B[1;1H");
                println!("{}", content)
            }
            Err(e) => {
                print!("\x1B[2J\x1B[1;1H");
                eprintln!("Unable to read file {file_path:?}: {e}");
                exit(1);
            }
        }
        let duration = Duration::from_secs_f32(*seconds);
        thread::sleep(duration);
    }
}

fn main() {
    let config: Config = process_args();
    println!("Source: {:?}", config.dir);
    let script_values = parse_script(&config.dir);
    if config.cycle {
        loop {
            process_script(&config.dir, &script_values);
        }
    } else {
        process_script(&config.dir, &script_values);
    }
}
