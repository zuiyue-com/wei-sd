use std::env;
use serde_json::Value;
use serde_json::json;

#[macro_use]
extern crate wei_log;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    wei_windows::init();

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
            let client = reqwest::Client::new();
            let response = client.get("http://localhost:7860/sdapi/v1/progress?skip_current_image=false")
            .header("accept", "application/json")
            .header("Content-Type", "application/json")
            .send().await.unwrap();

            match response.text().await {
                Ok(data) => {
                    if data == "" {
                        print!("{}", json!({"code": 400}).to_string());
                        return Ok(());
                    }
                    print!("{}", json!({"code": 200}).to_string());
                },
                Err(_) => {
                    print!("{}", json!({"code": 400}).to_string());
                }
            }
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

    let action_path = &args[2];
    let payload_str = &args[3];
    let report_url_process = args[4].clone();

    // report_url_process 是一个url加modac=process&time={}&uuid={}&taskUuid={}的地址
    // 我需要把modac,time,uuid,taskUuid提取出来

    let url = report_url_process.clone();
    let re = regex::Regex::new(r"modac=(.*?)&time=(.*?)&uuid=(.*?)&taskUuid=(.*)").unwrap();
    let caps = re.captures(&url).unwrap();

    let modac = &caps[1];
    let time = &caps[2];
    let uuid = &caps[3];
    let task_uuid = &caps[4];

    let mut report_url_process_body = serde_json::json!({
        "time": time,
        "uuid": uuid,
        "modac": modac,
        "taskUuid": task_uuid
    });

    info!("payload_str: {}", payload_str);
    
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

            let data: Value = match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(_) => {
                    break;
                }
            };

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

            report_url_process_body["info"] = data;
            
            info!("report_url_process_body: {}", report_url_process_body);

            client.post(report_url_process.clone())
                .body(report_url_process_body.to_string()).send().await.unwrap();

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    let client = reqwest::Client::new();
    let url = format!("http://localhost:7860{}", action_path);

    let response = match client.post(url)
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await {
            Ok(v) => v,
            Err(e) => {
                print!("{}", json!({
                    "code": 400,
                    "message": format!("Error:{}", e)
                }).to_string());
                handle.abort();
                return Ok(());
            }
    };

    let data = base64::encode(response.text().await?);

    if data == "" {
        print!("{}", json!({
            "code": 400,
            "message": "Error: Empty response"
        }).to_string());
    } else {
        print!("{}", json!({
            "code": 200,
            "message": "Sd-Ok",
            "data": data
        }).to_string());
    }

    handle.abort();

    Ok(())
}