use std::io::{self, Read};

use anyhow::Context;
use comfyui_api::comfy::{Comfy, ImageInfo, NodeOutput, PromptBuilder};
use futures_util::StreamExt;
use tokio::pin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut prompt = String::new();
    io::stdin()
        .read_to_string(&mut prompt)
        .context("failed to read prompt")?;

    let prompt = serde_json::from_str(prompt.as_str()).unwrap();
    let comfy = Comfy::new()?;

    let stream = comfy.stream_prompt(&prompt).await?;

    pin!(stream);

    while let Some(image) = stream.next().await {
        match image {
            Ok(NodeOutput { node, .. }) => {
                let image_info = ImageInfo::new_from_prompt(&prompt, &node)?;
                println!("Generated image: {:#?}.", image_info);
            }
            Err(err) => println!("Error: {:?}", err),
        }
    }

    let new_prompt = PromptBuilder::new(&prompt, None).seed(1, None).build()?;

    let images = comfy.execute_prompt(&new_prompt).await?;

    println!("Generated {} images.", images.len());

    for NodeOutput { node, .. } in images {
        let image_info = ImageInfo::new_from_prompt(&new_prompt, &node)?;
        println!("Generated image: {:#?}.", image_info);
    }

    Ok(())
}
