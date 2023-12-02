use std::io::{self, Read};

use anyhow::Context;
use comfyui_api::comfy::Comfy;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = String::new();
    io::stdin()
        .read_to_string(&mut prompt)
        .context("failed to read prompt")?;

    let prompt = serde_json::from_str(prompt.as_str()).unwrap();
    let comfy = Comfy::new()?;

    let mut stream = comfy.stream_prompt(&prompt).await?.boxed();
    while let Some(image) = stream.next().await {
        match image {
            Ok(_image) => println!("Generated image."),
            Err(err) => println!("Error: {:?}", err),
        }
    }

    let images = comfy.execute_prompt(&prompt).await?;

    println!("Generated {} images.", images.len());

    Ok(())
}
