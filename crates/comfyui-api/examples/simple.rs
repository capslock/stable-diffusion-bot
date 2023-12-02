use std::io::{self, stdout, Read, Write};

use anyhow::Context;
use futures_util::stream::StreamExt;

use comfyui_api::api::Api;
use comfyui_api::models::{NodeOutputOrUnknown, Prompt, Update};
use tokio::pin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = String::new();
    io::stdin()
        .read_to_string(&mut prompt)
        .context("failed to read prompt")?;

    let prompt: Prompt = serde_json::from_str(prompt.as_str()).unwrap();

    let api = Api::default();
    println!("API created with default host/port");
    let prompt_api = api.prompt()?;
    println!("Prompt API created");
    let history = api.history()?;
    println!("History API created");
    let view_api = api.view()?;
    println!("View API created");

    let websocket = api.websocket()?;
    println!("Websocket API created");
    let stream = websocket.updates().await?;

    println!("Sending prompt...");
    let response = prompt_api.send(&prompt).await?;

    println!("Prompt sent, id: {}", response.prompt_id);
    println!("Waiting for updates...");

    pin!(stream);

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(msg) => match msg {
                Update::ExecutionStart(data) => {
                    println!("Execution started: {:?}", data);
                }
                Update::Executing(data) => {
                    if let Some(node) = data.node {
                        println!("Executing: {:#?}", prompt.workflow[&node]);
                    } else if let Some(ref prompt_id) = data.prompt_id {
                        println!("Nothing left to execute.");
                        let task = history.get_prompt(prompt_id).await?;
                        println!("Number: {}", task.prompt.num);
                        for (key, value) in task.outputs.nodes.iter() {
                            if let NodeOutputOrUnknown::NodeOutput(output) = value {
                                println!("Node: {}", key);
                                for image in output.images.iter() {
                                    println!("Generated image: {:?}", image);
                                }
                            }
                        }
                        break;
                    }
                }
                Update::ExecutionCached(data) => {
                    println!("Execution cached: {:?}", data.nodes);
                }
                Update::Executed(data) => {
                    let _image = view_api.get(&data.output.images[0]).await?;
                    for image in data.output.images.iter() {
                        println!("Generated image: {:?}", image);
                    }
                }
                Update::ExecutionInterrupted(data) => {
                    println!("Execution interrupted: {:#?}", data);
                    break;
                }
                Update::ExecutionError(data) => {
                    println!("Execution error: {:#?}", data);
                    break;
                }
                Update::Progress(data) => {
                    if data.value == data.max {
                        println!(".")
                    } else {
                        print!(".");
                        stdout().flush().context("failed to flush stdout")?;
                    }
                }
                Update::Status { status } => {
                    println!("Status: {} queued.", status.exec_info.queue_remaining);
                }
            },
            Err(e) => {
                println!("Error occurred: {:#?}", e);
            }
        }
    }

    Ok(())
}
