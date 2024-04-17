extern crate dotenv;

use dotenv::dotenv;
use anyhow::Result;
use std::env;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct CommandRequest {
    name: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    let config = aws_config::load_from_env().await;
    let ssm = aws_sdk_ssm::Client::new(&config);
    let id_key = env::var("BOT_ID")?;
    let token_key = env::var("BOT_TOKEN")?;

    let params = ssm.get_parameters()
        .set_names(Some(vec![id_key, token_key]))
        .with_decryption(true)
        .send()
        .await?;

    //println!("{:?}", params.parameters());
    let mut url: String = "".to_owned();
    let mut token: String = "".to_owned();

    for param in params.parameters().iter() {
        let param_name = param.name().unwrap();
        let param_val = param.value().unwrap();
        if param_name.contains("ID") {
            url = format!("https://discord.com/api/v10/applications/{}/commands", param_val);
        }
        if param_name.contains("TOKEN") {
            token = param_val.to_owned();
        }
    }

    let client = reqwest::Client::new();
    let request_body: CommandRequest = CommandRequest{
        name: "space".to_string(),
        description: "get daily astronomy picture from nasa!".to_string(),
    };
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Bot {}", token).parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());


    let res = client.post(url)
        .body(serde_json::to_string(&request_body).unwrap())
        .headers(headers)
        .send()
        .await;
    println!("{:?}", res);
    // make request to register commands
    Ok(())
}
