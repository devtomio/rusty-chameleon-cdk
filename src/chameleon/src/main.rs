use dryoc::classic;
use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
extern crate url;
use url::form_urlencoded;

fn verify_key(
    body: &[u8],
    signature: &[u8],
    timestamp: &[u8],
    public_key: &[u8],
) -> Result<bool, Error> {
    let message = [timestamp, body].concat();
    let mut sig = [0u8; 64];
    let mut pk = [0u8; 32];

    hex::decode_to_slice(signature, &mut sig)?;
    hex::decode_to_slice(public_key, &mut pk)?;

    if classic::crypto_sign::crypto_sign_verify_detached(&sig, &message, &pk).is_ok() {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DiscordData {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomBody {
    data: DiscordData,
    #[serde(rename = "type")]
    kind: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageData {
    url: String,
    height: i32,
    width: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbedData {
    #[serde(rename = "type")]
    kind: String,
    title: String,
    color: i32,
    image: ImageData,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbedResponseData {
    content: String,
    embeds: Vec<EmbedData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResponseData {
    content: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct CustomResponse {
    #[serde(rename = "type")]
    kind: i64,
    data: ResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomEmbedResponse {
    #[serde(rename = "type")]
    kind: i64,
    data: EmbedResponseData,
}

async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let config = aws_config::load_from_env().await;
    let ssm = aws_sdk_ssm::Client::new(&config);
    println!("got event {:?}", &event.body());
    let signature = event
        .headers()
        .get("X-Signature-Ed25519")
        .unwrap()
        .to_str()?;

    let timestamp = event
        .headers()
        .get("X-Signature-Timestamp")
        .unwrap()
        .to_str()?;

    // get PK and NASA API KEY from SSM
    let public_key = "/rusty-chameleon/public-key".to_string();
    let nasa_api_key = "NASA_API_KEY".to_string();

    println!("grabing pk and nasa api key");
    let public_key = ssm
        .get_parameter()
        .set_name(Some(public_key.clone()))
        .with_decryption(true)
        .send()
        .await
        .unwrap()
        .parameter
        .unwrap()
        .value
        .unwrap();

    let nasa_api_key = ssm
        .get_parameter()
        .set_name(Some(nasa_api_key.clone()))
        .with_decryption(true)
        .send()
        .await
        .unwrap()
        .parameter
        .unwrap()
        .value
        .unwrap();

    println!("verifying key");

    Ok(
        match verify_key(
            event.body(),
            signature.as_bytes(),
            timestamp.as_bytes(),
            public_key.as_bytes(),
        ) {
            Ok(ok) if ok => {
                println!("Building OK response");
                let body: CustomBody = serde_json::from_slice(&event.body() as &[u8]).unwrap();
                // let body: CustomBody = serde_json::from_str(&event.body() as String).unwrap()?;
                if &body.kind == &1i64 {
                    println!("Received Ping for ack");
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body("{ \"type\": 1 }".to_string())
                        .map_err(Box::new)?
                } else if &body.kind == &2i64 {
                    println!("Application command received");
                    let command_name: String = body.data.name;
                    println!("command_name: {:?}", command_name);

                    if command_name == "foo" {
                        println!("command foo activated");
                        let res = CustomResponse {
                            kind: 4,
                            data: ResponseData {
                                content: "bar".to_owned(),
                            },
                        };
                        return Ok(Response::builder()
                            .status(200)
                            .header("content-type", "application/json")
                            .body(serde_json::to_string(&res).unwrap())
                            .map_err(Box::new)?);
                    }
                    if command_name == "space" {
                        return space_cmd_handler(&nasa_api_key).await;
                    }

                    // methods for different types of application commands
                    // can probably leverage serenity constructs here
                    // id	        snowflake	                                                the ID of the invoked command
                    // name	        string	                                                    the name of the invoked command
                    // type	        integer	                                                    the type of the invoked command
                    // resolved?	resolved data	                                            converted users + roles + channels + attachments
                    // options?*	array of application command interaction data option	    the params + values from the user
                    // guild_id?	snowflake	                                                the id of the guild the command is registered to
                    // target_id?	snowflake	                                                id of the user or message targeted by a user or message command

                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body("{ \"type\": 1 }".to_string())
                        .map_err(Box::new)?
                } else if &body.kind == &3i64 {
                    println!("Message component received");
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body("{ \"type\": 1 }".to_string())
                        .map_err(Box::new)?
                } else if &body.kind == &4i64 {
                    println!("Application command autocomplete received");
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body("{ \"type\": 1 }".to_string())
                        .map_err(Box::new)?
                } else if &body.kind == &5i64 {
                    println!("Modal submit received");
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body("{ \"type\": 1 }".to_string())
                        .map_err(Box::new)?
                } else {
                    println!("Unknown interaction type received, sending default response");
                    Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .body("{ \"type\": 1 }".to_string())
                        .map_err(Box::new)?
                }
            }
            Ok(_) => Response::builder()
                .status(401)
                .body("Invalid request signature".to_string())?,
            Err(e) => {
                println!("An unknown error occured {:?}", e);

                Response::builder()
                    .status(500)
                    .body(format!("An unknown error occured {:?}", e))?
            }
        },
    )
}

#[derive(Serialize, Deserialize, Debug)]
struct NasaApiResponseData {
    url: String,
    title: String,
    explanation: String,
}

async fn space_cmd_handler(nasa_api_key: &str) -> Result<Response<String>, Error> {
    // make request to nasa API

    let client = reqwest::Client::new();
    let reqwest_res = client
            .get(format!(
                "https://api.nasa.gov/planetary/apod?api_key={}",
                nasa_api_key
            ))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

    println!("response {:?}", &reqwest_res);
    let nasa_res: NasaApiResponseData = serde_json::from_str(&reqwest_res)
    .unwrap();
    // destructure response
    //
    let encoded_url = url::Url::parse(&nasa_res.url).unwrap().to_string();
    println!("returning with encoded url: {:?}", &encoded_url);

    let res = CustomEmbedResponse {
        kind: 4,
        data: EmbedResponseData {
            content: nasa_res.explanation,
            embeds: vec![EmbedData {
                kind: "rich".to_owned(),
                title: nasa_res.title,
                color: 0x00FFFF,
                image: ImageData {
                    url: encoded_url,
                    height: 0,
                    width: 0,
                },
            }],
        },
    };

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&res).unwrap())
        .map_err(Box::new)?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
