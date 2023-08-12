use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        help();
        std::process::exit(1);
    }
    let command = &args[1];

    match command.as_str() {
        "install" => {
            println!("Installing...");
        },
        "uninstall" => {
            println!("Uninstalling...");
        },
        "api" => {
            let json = &args[2];
            let value: serde_json::Value = serde_json::from_str(json).unwrap();
            println!("{:#?}", value);
        },
        _ => {
            help();
            std::process::exit(1);
        }
    }
}

fn help() {
    let args: Vec<String> = env::args().collect();
    eprintln!("Usage:");
    eprintln!("  {} install", args[0]);
    eprintln!("  {} uninstall", args[0]);
    eprintln!("  {} api <json>", args[0]);
}