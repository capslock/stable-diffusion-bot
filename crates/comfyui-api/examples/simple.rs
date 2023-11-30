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
    let prompt = serde_json::from_str::<Prompt>(r#"{
  "5": {
    "inputs": {
      "width": 1024,
      "height": 1024,
      "batch_size": 4
    },
    "class_type": "EmptyLatentImage"
  },
  "6": {
    "inputs": {
      "text": "a water color of a corgi wearing a tophat",
      "clip": [
        "20",
        1
      ]
    },
    "class_type": "CLIPTextEncode"
  },
  "7": {
    "inputs": {
      "text": "text, watermark, ugly, worst quality",
      "clip": [
        "20",
        1
      ]
    },
    "class_type": "CLIPTextEncode"
  },
  "8": {
    "inputs": {
      "samples": [
        "13",
        0
      ],
      "vae": [
        "20",
        2
      ]
    },
    "class_type": "VAEDecode"
  },
  "13": {
    "inputs": {
      "add_noise": true,
      "noise_seed": 0,
      "cfg": 1,
      "model": [
        "20",
        0
      ],
      "positive": [
        "6",
        0
      ],
      "negative": [
        "7",
        0
      ],
      "sampler": [
        "14",
        0
      ],
      "sigmas": [
        "22",
        0
      ],
      "latent_image": [
        "5",
        0
      ]
    },
    "class_type": "SamplerCustom"
  },
  "14": {
    "inputs": {
      "sampler_name": "dpmpp_3m_sde_gpu"
    },
    "class_type": "KSamplerSelect"
  },
  "20": {
    "inputs": {
      "ckpt_name": "downloaded\\sdxl-turbo\\turbovisionxlSuperFastXLBasedOnNew_alphaV0101Bakedvae.safetensors"
    },
    "class_type": "CheckpointLoaderSimple"
  },
  "22": {
    "inputs": {
      "steps": 4,
      "model": [
        "20",
        0
      ]
    },
    "class_type": "SDTurboScheduler"
  },
  "25": {
    "inputs": {
      "images": [
        "8",
        0
      ]
    },
    "class_type": "PreviewImage"
  }
}"#).unwrap();
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
            println!("{:#?}", msg);
        })
        .await;

    Ok(())
}
