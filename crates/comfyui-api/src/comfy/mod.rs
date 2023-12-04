use std::collections::HashSet;
use std::pin::pin;

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
        let mut stream = pin!(self.stream_prompt(prompt).await?);
        while let Some(image) = stream.next().await {
            match image {
                Ok(image) => images.push(image),
                Err(e) => return Err(e),
            }
        }
        Ok(images)
    }
}

/// Information about the generated image.
#[derive(Debug, Clone, Default)]
pub struct ImageInfo {
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub model: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub seed: Option<i64>,
}

impl ImageInfo {
    /// Returns a new `ImageInfo` instance based on the given `Prompt` and output node.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A `Prompt` describing the workflow used to generate an image.
    /// * `output_node` - The output node that produced the image.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `ImageInfo` instance on success, or an error if the output node was not found.
    pub fn new_from_prompt(prompt: &Prompt, output_node: &str) -> anyhow::Result<ImageInfo> {
        let mut image_info = ImageInfo::default();
        if let Some(node) = prompt.get_node_by_id(output_node) {
            image_info.visit(prompt, node);
        } else {
            return Err(anyhow!("Output node not found: {}", output_node));
        }
        Ok(image_info)
    }
}
