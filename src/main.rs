use std::env;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    wei_env::bin_init("wei-sd");
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
        "check" => {

        },
        "api" => {
            api().await?;
        },
        _ => {
            help();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn help() {
    let args: Vec<String> = env::args().collect();
    println!("Usage:");
    println!("  {} install", args[0]);
    println!("  {} uninstall", args[0]);
    println!("  {} api <url> <json>", args[0]);
}

async fn api() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    let payload_str = &args[3];
    
    // 尝试将参数解析为 JSON
    let payload: Value = match serde_json::from_str(payload_str) {
        Ok(v) => v,
        Err(e) => {
            print!("{{\"code\": 400,\"status\": \"Error:{} Payload:{}\"}}", e, payload_str);
            return Ok(());
        }
    };

    let client = reqwest::Client::new();
    let url = format!("http://192.168.1.8:7860{}", &args[2]);

    let response = client.post(url)
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    print!("{{\"code\": 200,\"status\": \"Ok\", \"data\": {:?}}}", response.text().await?);

    Ok(())
}