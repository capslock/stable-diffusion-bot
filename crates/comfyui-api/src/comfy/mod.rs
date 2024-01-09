use std::collections::HashSet;
use std::pin::pin;

use anyhow::{anyhow, Context};
use async_stream::stream;
use futures_util::{
    stream::{FusedStream, FuturesOrdered},
    Stream, StreamExt,
};
use uuid::Uuid;

use crate::{api::*, models::*};

pub mod visitor;
pub use visitor::Visitor;

pub mod setter;

pub mod getter;
use getter::*;

mod accessors;

use self::setter::SetterExt as _;

enum State {
    Executing(String, Vec<Image>),
    Finished(Vec<(String, Vec<Image>)>),
}

/// Output from a node.
#[derive(Debug, Clone)]
pub struct NodeOutput {
    /// The identifier of the node.
    pub node: String,
    /// The image generated by the node.
    pub image: Vec<u8>,
}

/// Higher-level API for interacting with the ComfyUI API.
#[derive(Clone, Debug)]
pub struct Comfy {
    api: Api,
    history: HistoryApi,
    upload: UploadApi,
    view: ViewApi,
}

impl Default for Comfy {
    fn default() -> Self {
        let api = Api::default();
        Self {
            history: api.history().expect("failed to create history api"),
            upload: api.upload().expect("failed to create upload api"),
            view: api.view().expect("failed to create view api"),
            api,
        }
    }
}

impl Comfy {
    /// Returns a new `Comfy` instance with default settings.
    pub fn new() -> anyhow::Result<Self> {
        let api = Api::default();
        Ok(Self {
            history: api.history()?,
            upload: api.upload()?,
            view: api.view()?,
            api,
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
            history: api.history()?,
            upload: api.upload()?,
            view: api.view()?,
            api,
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
            history: api.history()?,
            upload: api.upload()?,
            view: api.view()?,
            api,
        })
    }

    async fn filter_update(
        &self,
        update: Update,
        target_prompt_id: Uuid,
    ) -> anyhow::Result<Option<State>> {
        match update {
            Update::Executing(data) => {
                if data.node.is_none() {
                    if let Some(prompt_id) = data.prompt_id {
                        if prompt_id != target_prompt_id {
                            return Ok(None);
                        }
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
            Update::Executed(data) => {
                if data.prompt_id != target_prompt_id {
                    return Ok(None);
                }
                Ok(Some(State::Executing(data.node, data.output.images)))
            }
            Update::ExecutionInterrupted(data) => {
                if data.prompt_id != target_prompt_id {
                    return Ok(None);
                }
                Err(anyhow::anyhow!("Execution interrupted: {:?}", data))
            }
            Update::ExecutionError(data) => {
                if data.execution_status.prompt_id != target_prompt_id {
                    return Ok(None);
                }
                Err(anyhow::anyhow!("Execution error: {:?}", data))
            }
            _ => Ok(None),
        }
    }

    async fn prompt_impl<'a>(
        &'a self,
        prompt: &'a Prompt,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<State>> + 'a> {
        let client_id = Uuid::new_v4();
        let prompt_api = self.api.prompt_with_client(client_id)?;
        let websocket_api = self.api.websocket_with_client(client_id)?;
        let stream = websocket_api.updates().await?;
        let response = prompt_api.send(prompt).await?;
        let prompt_id = response.prompt_id;
        Ok(stream.filter_map(move |msg| async move {
            match msg {
                Ok(msg) => match self.filter_update(msg, prompt_id).await {
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
    /// A `Result` containing a `Stream` of `Result<(String, Vec<u8>)>` values on success, or an error if the request failed.
    /// The `String` value is the output node name, and the `Vec<u8>` value is the image data.
    pub async fn stream_prompt<'a>(
        &'a self,
        prompt: &'a Prompt,
    ) -> anyhow::Result<impl FusedStream<Item = anyhow::Result<NodeOutput>> + 'a> {
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
                            yield Ok(NodeOutput { node: node.clone(), image: image? });
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
                                yield Ok(NodeOutput { node: node.clone(), image: image? });
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
    /// A `Result` containing a pair of `String` and `Vec<u8>` values on success, or an error if the request failed.
    /// The `String` value is the output node name, and the `Vec<u8>` value is the image data.
    pub async fn execute_prompt(&self, prompt: &Prompt) -> anyhow::Result<Vec<NodeOutput>> {
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

    pub async fn upload_file(&self, file: Vec<u8>) -> anyhow::Result<ImageUpload> {
        self.upload
            .image(file)
            .await
            .context("failed to upload file")
    }
}

/// Information about the generated image.
#[derive(Debug, Clone, Default)]
pub struct ImageInfo {
    /// The prompt used to generate the image.
    pub prompt: Option<String>,
    /// The negative prompt used to generate the image.
    pub negative_prompt: Option<String>,
    /// The model used to generate the image.
    pub model: Option<String>,
    /// The width of the image.
    pub width: Option<u32>,
    /// The height of the image.
    pub height: Option<u32>,
    /// The seed used to generate the image.
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

#[derive(Debug, Clone)]
struct OverrideNode<T> {
    node: Option<String>,
    value: T,
}

impl<T> Default for OverrideNode<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            node: Default::default(),
            value: Default::default(),
        }
    }
}

/// A builder for creating a `Prompt` instance.
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    base_prompt: Prompt,
    output_node: Option<String>,
    prompt: Option<OverrideNode<String>>,
    negative_prompt: Option<OverrideNode<String>>,
    model: Option<OverrideNode<String>>,
    width: Option<OverrideNode<u32>>,
    height: Option<OverrideNode<u32>>,
    seed: Option<OverrideNode<i64>>,
}

impl PromptBuilder {
    /// Constructs a new `PromptBuilder` instance.
    ///
    /// # Arguments
    ///
    /// * `base_prompt` - The base `Prompt` to use as a starting point.
    /// * `output_node` - The output node to use when building the prompt.
    ///
    /// # Returns
    ///
    /// A new `PromptBuilder` instance.
    pub fn new(base_prompt: &Prompt, output_node: Option<String>) -> Self {
        Self {
            prompt: None,
            negative_prompt: None,
            model: None,
            width: None,
            height: None,
            seed: None,
            base_prompt: base_prompt.clone(),
            output_node,
        }
    }

    /// Sets the prompt.
    ///
    /// # Arguments
    ///
    /// * `value` - The prompt value to use.
    /// * `node` - The node to set the prompt on.
    pub fn prompt(mut self, value: String, node: Option<String>) -> Self {
        self.prompt = Some(OverrideNode { node, value });
        self
    }

    /// Sets the negative prompt.
    ///
    /// # Arguments
    ///
    /// * `value` - The negative prompt value to use.
    /// * `node` - The node to set the negative prompt on.
    pub fn negative_prompt(mut self, value: String, node: Option<String>) -> Self {
        self.negative_prompt = Some(OverrideNode { node, value });
        self
    }

    /// Sets the model.
    ///
    /// # Arguments
    ///
    /// * `value` - The model value to use.
    /// * `node` - The node to set the model on.
    pub fn model(mut self, value: String, node: Option<String>) -> Self {
        self.model = Some(OverrideNode { node, value });
        self
    }

    /// Sets the width.
    ///
    /// # Arguments
    ///
    /// * `value` - The width value to use.
    /// * `node` - The node to set the width on.
    pub fn width(mut self, value: u32, node: Option<String>) -> Self {
        self.width = Some(OverrideNode { node, value });
        self
    }

    /// Sets the height.
    ///
    /// # Arguments
    ///
    /// * `value` - The height value to use.
    /// * `node` - The node to set the height on.
    pub fn height(mut self, value: u32, node: Option<String>) -> Self {
        self.height = Some(OverrideNode { node, value });
        self
    }

    /// Sets the seed.
    ///
    /// # Arguments
    ///
    /// * `value` - The seed value to use.
    /// * `node` - The node to set the seed on.
    pub fn seed(mut self, value: i64, node: Option<String>) -> Self {
        self.seed = Some(OverrideNode { node, value });
        self
    }

    /// Builds a new `Prompt` instance based on the given parameters.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `Prompt` instance on success, or an error if a suitable output node could not be found.
    pub fn build(mut self) -> anyhow::Result<Prompt> {
        let mut new_prompt = self.base_prompt.clone();

        if self.output_node.is_none() {
            self.output_node = Some(
                find_output_node(&new_prompt).context("failed to find a suitable output node")?,
            );
        }

        if let Some(ref prompt) = self.prompt {
            if let Some(ref node) = prompt.node {
                new_prompt.set_node::<accessors::Prompt>(node, prompt.value.clone())?;
            } else {
                new_prompt.set_from::<accessors::Prompt>(
                    &self.output_node.clone().unwrap(),
                    prompt.value.clone(),
                )?;
            }
        }
        if let Some(ref negative_prompt) = self.negative_prompt {
            if let Some(ref node) = negative_prompt.node {
                new_prompt
                    .set_node::<accessors::NegativePrompt>(node, negative_prompt.value.clone())?;
            } else {
                new_prompt.set_from::<accessors::NegativePrompt>(
                    &self.output_node.clone().unwrap(),
                    negative_prompt.value.clone(),
                )?;
            }
        }
        if let Some(ref model) = self.model {
            if let Some(ref node) = model.node {
                new_prompt.set_node::<accessors::Model>(node, model.value.clone())?;
            } else {
                new_prompt.set_from::<accessors::Model>(
                    &self.output_node.clone().unwrap(),
                    model.value.clone(),
                )?;
            }
        }
        if let Some(width) = self.width {
            if let Some(ref node) = width.node {
                new_prompt.set_node::<accessors::Width>(node, width.value)?;
            } else {
                new_prompt.set_from::<accessors::Width>(
                    &self.output_node.clone().unwrap(),
                    width.value,
                )?;
            }
        }
        if let Some(height) = self.height {
            if let Some(ref node) = height.node {
                new_prompt.set_node::<accessors::Height>(node, height.value)?;
            } else {
                new_prompt.set_from::<accessors::Height>(
                    &self.output_node.clone().unwrap(),
                    height.value,
                )?;
            }
        }
        if let Some(ref seed) = self.seed {
            if let Some(ref node) = seed.node {
                new_prompt.set_node::<accessors::Seed>(node, seed.value)?;
            } else {
                new_prompt
                    .set_from::<accessors::Seed>(&self.output_node.clone().unwrap(), seed.value)?;
            }
        }
        Ok(new_prompt)
    }
}
