use std::collections::HashSet;

use anyhow::anyhow;
use async_stream::stream;
use futures_util::{
    stream::{FusedStream, FuturesOrdered},
    Stream, StreamExt,
};

use crate::{api::*, models::*};

pub mod visitor;
pub use visitor::Visitor;

/// Higher-level API for interacting with the ComfyUI API.
#[derive(Clone, Debug)]
pub struct Comfy {
    prompt: PromptApi,
    view: ViewApi,
    websocket: WebsocketApi,
    history: HistoryApi,
}

enum State {
    Executing(String, Vec<Image>),
    Finished(Vec<(String, Vec<Image>)>),
}

impl Comfy {
    /// Returns a new `Comfy` instance with default settings.
    pub fn new() -> anyhow::Result<Self> {
        let api = Api::default();
        Ok(Self {
            prompt: api.prompt()?,
            view: api.view()?,
            websocket: api.websocket()?,
            history: api.history()?,
        })
    }

    /// Returns a new `Comfy` instance with the given URL as a string value.
    ///
    /// # Arguments
    ///
    /// * `url` - A string that specifies the ComfyUI API URL endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn new_with_url<S>(url: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        let api = Api::new_with_url(url.as_ref())?;
        Ok(Self {
            prompt: api.prompt()?,
            view: api.view()?,
            websocket: api.websocket()?,
            history: api.history()?,
        })
    }

    /// Returns a new `Comfy` instance with the given `reqwest::Client` and URL as a string value.
    ///
    /// # Arguments
    ///
    /// * `client` - An instance of `reqwest::Client`.
    /// * `url` - A string that specifies the ComfyUI API URL endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn new_with_client_and_url<S>(client: reqwest::Client, url: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        let api = Api::new_with_client_and_url(client, url.as_ref())?;
        Ok(Self {
            prompt: api.prompt()?,
            view: api.view()?,
            websocket: api.websocket()?,
            history: api.history()?,
        })
    }

    async fn filter_update(&self, update: Update) -> anyhow::Result<Option<State>> {
        match update {
            Update::Executing(data) => {
                if data.node.is_none() {
                    if let Some(prompt_id) = data.prompt_id {
                        let task = self.history.get_prompt(&prompt_id).await.unwrap();
                        let images = task
                            .outputs
                            .nodes
                            .into_iter()
                            .filter_map(|(key, value)| {
                                if let NodeOutputOrUnknown::NodeOutput(output) = value {
                                    Some((key, output.images))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<(String, Vec<Image>)>>();
                        return Ok(Some(State::Finished(images)));
                    }
                }
                Ok(None)
            }
            Update::Executed(data) => Ok(Some(State::Executing(data.node, data.output.images))),
            Update::ExecutionInterrupted(data) => {
                Err(anyhow::anyhow!("Execution interrupted: {:?}", data))
            }
            Update::ExecutionError(data) => Err(anyhow::anyhow!("Execution error: {:?}", data)),
            _ => Ok(None),
        }
    }

    async fn prompt_impl<'a>(
        &'a self,
        prompt: &'a Prompt,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<State>> + 'a> {
        let stream = self.websocket.updates().await?;
        let _response = self.prompt.send(prompt).await?;
        Ok(stream.filter_map(move |msg| async {
            match msg {
                Ok(msg) => match self.filter_update(msg).await {
                    Ok(Some(images)) => Some(Ok(images)),
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                },
                Err(e) => Some(Err(anyhow::anyhow!("Error occurred: {:?}", e))),
            }
        }))
    }

    /// Executes a prompt and returns a stream of generated images.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A `Prompt` to send to the ComfyUI API.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Stream` of `Result<Vec<u8>>` values on success, or an error if the request failed.
    pub async fn stream_prompt<'a>(
        &'a self,
        prompt: &'a Prompt,
    ) -> anyhow::Result<impl FusedStream<Item = anyhow::Result<(String, Vec<u8>)>> + 'a> {
        Ok(stream! {
            let mut executed = HashSet::new();
            let stream = self.prompt_impl(prompt).await?;
            for await msg in stream {
                match msg {
                    Ok(State::Executing(node, images)) => {
                        executed.insert(node.clone());
                        let fut = images.into_iter().map(|image| async move {
                            self.view.get(&image).await
                        }).collect::<FuturesOrdered<_>>();
                        for await image in fut {
                            yield Ok((node.clone(), image?));
                        }
                    }
                    Ok(State::Finished(images)) => {
                        for (node, images) in images {
                            if executed.contains(&node) {
                                continue;
                            }
                            let fut = images.into_iter().map(|image| async move {
                                self.view.get(&image).await
                            }).collect::<FuturesOrdered<_>>();
                            for await image in fut {
                                yield Ok((node.clone(), image?));
                            }
                        }
                        return;
                    }
                    Err(e) => Err(e)?,
                }
            }
        })
    }

    /// Executes a prompt and returns the generated images.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A `Prompt` to send to the ComfyUI API.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<Vec<u8>>` of images on success, or an error if the request failed.
    pub async fn execute_prompt(&self, prompt: &Prompt) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
        let mut images = vec![];
        let mut stream = self.stream_prompt(prompt).await?.boxed();
        while let Some(image) = stream.next().await {
            match image {
                Ok(image) => images.push(image),
                Err(e) => return Err(e),
            }
        }
        Ok(images)
    }
}

impl<'a> Visitor for ImageInfoVisitor<'a> {
    fn get_node_by_id(&self, id: &str) -> &NodeOrUnknown {
        &self.prompt.workflow[id]
    }

    fn visit_clip_text_encode(&mut self, node: &CLIPTextEncode) {
        if self.prompt_text.is_none() {
            if let Input::Value(text) = &node.text {
                self.prompt_text = Some(text.clone());
            }
        } else if self.negative_prompt_text.is_none() {
            if let Input::Value(text) = &node.text {
                self.negative_prompt_text = Some(text.clone());
            }
        }
        visitor::walk_clip_text_encode(self, node)
    }

    fn visit_checkpoint_loader_simple(&mut self, node: &CheckpointLoaderSimple) {
        if let Input::Value(model) = &node.ckpt_name {
            self.model = Some(model.clone());
        }
        visitor::walk_checkpoint_loader_simple(self, node)
    }

    fn visit_image_only_checkpoint_loader(&mut self, node: &ImageOnlyCheckpointLoader) {
        if let Input::Value(model) = &node.ckpt_name {
            self.model = Some(model.clone());
        }
        visitor::walk_image_only_checkpoint_loader(self, node)
    }

    fn visit_empty_latent_image(&mut self, node: &EmptyLatentImage) {
        if let Input::Value(width) = &node.width {
            self.width = Some(*width);
        }
        if let Input::Value(height) = &node.height {
            self.height = Some(*height);
        }
        visitor::walk_empty_latent_image(self, node)
    }

    fn visit_k_sampler(&mut self, node: &KSampler) {
        if let Input::Value(seed) = &node.seed {
            self.seed = Some(*seed);
        }
        visitor::walk_k_sampler(self, node)
    }

    fn visit_sampler_custom(&mut self, node: &SamplerCustom) {
        if let Input::Value(seed) = &node.noise_seed {
            self.seed = Some(*seed);
        }
        visitor::walk_sampler_custom(self, node)
    }
}

struct ImageInfoVisitor<'a> {
    pub prompt: &'a Prompt,
    pub prompt_text: Option<String>,
    pub negative_prompt_text: Option<String>,
    pub model: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub seed: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub prompt: String,
    pub negative_prompt: String,
    pub width: u32,
    pub height: u32,
    pub model: String,
    pub seed: i64,
}

impl ImageInfo {
    pub fn new_from_prompt(prompt: &Prompt, output_node: &str) -> anyhow::Result<ImageInfo> {
        if prompt.workflow.get(output_node).is_none() {
            return Err(anyhow!("Output node not found: {}", output_node));
        }
        let mut visitor = ImageInfoVisitor {
            prompt,
            prompt_text: None,
            negative_prompt_text: None,
            model: None,
            width: None,
            height: None,
            seed: None,
        };
        visitor.visit_node_or_unknown(&visitor.get_node_by_id(output_node).clone());
        Ok(ImageInfo {
            prompt: visitor.prompt_text.unwrap_or_default(),
            negative_prompt: visitor.negative_prompt_text.unwrap_or_default(),
            width: visitor.width.unwrap_or_default(),
            height: visitor.height.unwrap_or_default(),
            model: visitor.model.unwrap_or_default(),
            seed: visitor.seed.unwrap_or_default(),
        })
    }
}
