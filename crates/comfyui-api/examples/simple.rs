use comfyui_api::{Api, Prompt, UpdateOrUnknown};
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let prompt: Prompt = serde_json::from_str(
        r#"
    {
        "3": {
            "class_type": "KSampler",
            "inputs": {
                "cfg": 8,
                "denoise": 1,
                "latent_image": [
                    "5",
                    0
                ],
                "model": [
                    "4",
                    0
                ],
                "negative": [
                    "7",
                    0
                ],
                "positive": [
                    "6",
                    0
                ],
                "sampler_name": "euler",
                "scheduler": "normal",
                "seed": 8566257,
                "steps": 20
            }
        },
        "4": {
            "class_type": "CheckpointLoaderSimple",
            "inputs": {
                "ckpt_name": "sd\\sd_xl_base_1.0.safetensors"
            }
        },
        "5": {
            "class_type": "EmptyLatentImage",
            "inputs": {
                "batch_size": 1,
                "height": 1024,
                "width": 1024
            }
        },
        "6": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "clip": [
                    "4",
                    1
                ],
                "text": "masterpiece best quality girl"
            }
        },
        "7": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "clip": [
                    "4",
                    1
                ],
                "text": "bad hands"
            }
        },
        "8": {
            "class_type": "VAEDecode",
            "inputs": {
                "samples": [
                    "3",
                    0
                ],
                "vae": [
                    "4",
                    2
                ]
            }
        },
        "9": {
            "class_type": "SaveImage",
            "inputs": {
                "filename_prefix": "ComfyUI",
                "images": [
                    "8",
                    0
                ]
            }
        }
    }
    "#,
    )
    .unwrap();
    println!("{:#?}", prompt);
    println!("{}", serde_json::to_string_pretty(&prompt).unwrap());

    let api = Api::default();
    let prompt_api = api.prompt()?;

    let websocket = api.websocket()?;
    let stream = websocket.connect().await?;

    let response = prompt_api.send(prompt).await?;
    println!("{:#?}", response);

    stream
        .for_each(|msg| async move {
            println!(
                "{:#?}",
                msg.map(
                    |m| serde_json::from_str::<UpdateOrUnknown>(&m.into_text().unwrap()).unwrap()
                )
            );
        })
        .await;

    Ok(())
}
