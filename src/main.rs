static DATA_1: &'static [u8] = include_bytes!("../../wei-test/r");

use std::env;
use serde_json::Value;
use serde_json::json;

#[macro_use]
extern crate wei_log;

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
        "data" => {
            println!("{:?}", DATA_1);
        },
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
    println!("  {} api <report_url_process> <url> <json>", args[0]);
}

async fn api() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    let report_url_process = args[2].clone();
    let report_url_process_body = args[3].clone();
    let action_path = &args[4];
    let payload_str = &args[5].replace("\\\"", "\"");
    
    // 尝试将参数解析为 JSON
    let payload: Value = match serde_json::from_str(payload_str) {
        Ok(v) => v,
        Err(e) => {
            print!("{}", json!({
                "code": 400,
                "message": format!("Error:{} Payload:{}", e, payload_str)
            }).to_string());
            return Ok(());
        }
    };

    // 开始任务进度报告
    let handle = tokio::spawn( async move {
        loop {
            let client = reqwest::Client::new();
            let response = client.get("http://localhost:7860/sdapi/v1/progress?skip_current_image=false")
                .header("accept", "application/json")
                .header("Content-Type", "application/json")
                .send().await.unwrap();

            let data = response.text().await.unwrap();

            let data: Value = serde_json::from_str(&data).unwrap();

            let data = serde_json::json!({
                "progress": data["progress"],
                "eta_relative": data["eta_relative"],
                "state": {
                    "skipped": data["state"]["skipped"],
                    "interrupted": data["state"]["interrupted"],
                    "job": data["state"]["job"],
                    "job_count": data["state"]["job_count"],
                    "job_timestamp": data["state"]["job_timestamp"],
                    "job_no": data["state"]["job_no"],
                    "sampling_step": data["state"]["sampling_step"],
                    "sampling_steps": data["state"]["sampling_steps"],
                },
                "textinfo": data["textinfo"]
            });

            let mut report_data = serde_json::from_str::<Value>(&report_url_process_body).unwrap();
            report_data["info"] = data;

            info!("info: {:?}", report_data);

            client.post(report_url_process.clone())
                .body(report_data.to_string()).send().await.unwrap();

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    let client = reqwest::Client::new();
    let url = format!("http://localhost:7860{}", action_path);

    let response = client.post(url)
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    print!("{}", json!({
        "code": 200,
        "message": "Ok",
        "data": base64::encode(response.text().await?)
    }).to_string());

    handle.abort();

    Ok(())
}