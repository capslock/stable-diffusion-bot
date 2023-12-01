use std::io::{self, stdout, Read, Write};

use anyhow::Context;
use futures_util::stream::StreamExt;

use comfyui_api::{Api, Prompt, Update};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = String::new();
    io::stdin()
        .read_to_string(&mut prompt)
        .context("failed to read prompt")?;

    let prompt: Prompt = serde_json::from_str(prompt.as_str()).unwrap();

    // println!("{:#?}", prompt);
    // println!("{}", serde_json::to_string_pretty(&prompt).unwrap());

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
    let mut stream = websocket.updates().await?;

    println!("Sending prompt...");
    let response = prompt_api.send(prompt).await?;

    println!("Prompt sent: {:#?}", response.prompt_id);
    // println!("{:#?}", response);
    println!("Waiting for updates...");

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(msg) => match msg {
                Update::Executed(data) => {
                    //println!("{:#?}", data);
                    let _image = view_api.get(&data.output.images[0]).await?;
                    println!("\nGenerated image: {:#?}", data.output.images[0]);
                    let task = history.get_prompt(&data.prompt_id).await?;
                    println!("Number: {}", task.prompt.num);
                    // println!(
                    //     "{:#?}\n{} bytes",
                    //     history.get(&data.prompt_id).await?,
                    //     image.len()
                    // )
                    break;
                }
                Update::ExecutionCached(data) => {
                    println!("\nExecution cached: {:#?}", data.nodes);
                    break;
                }
                _ => {
                    print!(".");
                    stdout().flush().context("failed to flush stdout")?;
                    //println!("{:#?}", msg);
                }
            },
            Err(e) => {
                println!("Error occurred: {:#?}", e);
            }
        }
    }

    Ok(())
}
