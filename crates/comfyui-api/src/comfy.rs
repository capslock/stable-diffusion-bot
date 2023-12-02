use anyhow::Context;
use futures_util::{future::join_all, StreamExt};

use crate::{api::*, models::*};

/// Struct representing a connection to a ComfyUI API.
#[derive(Clone, Debug)]
pub struct Comfy {
    prompt: PromptApi,
    view: ViewApi,
    websocket: WebsocketApi,
    history: HistoryApi,
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

    pub async fn execute_prompt(&self, prompt: &Prompt) -> anyhow::Result<Vec<Vec<u8>>> {
        let mut stream = self.websocket.updates().await?;
        let _response = self.prompt.send(prompt).await?;

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(msg) => match msg {
                    Update::ExecutionStart(_) => {}
                    Update::Executing(data) => {
                        if data.node.is_none() {
                            let task = self.history.get_prompt(&data.prompt_id).await?;
                            let images = task
                                .outputs
                                .nodes
                                .into_values()
                                .filter_map(|value| {
                                    if let NodeOutputOrUnknown::NodeOutput(output) = value {
                                        Some(output.images)
                                    } else {
                                        None
                                    }
                                })
                                .flatten()
                                .collect::<Vec<Image>>();
                            return join_all(
                                images
                                    .into_iter()
                                    .map(|image| async move { self.view.get(&image).await })
                                    .collect::<Vec<_>>(),
                            )
                            .await
                            .into_iter()
                            .try_fold(vec![], |mut acc, image| {
                                acc.push(image.context("Failed to get image")?);
                                Ok::<Vec<Vec<u8>>, anyhow::Error>(acc)
                            });
                        }
                    }
                    Update::ExecutionCached(_) => {}
                    Update::Executed(_data) => {
                        //let _image = self.view.get(&data.output.images[0]).await?;
                        //for image in data.output.images.iter() {
                        //    println!("Generated image: {:?}", image);
                        //}
                    }
                    Update::ExecutionInterrupted(data) => {
                        return Err(anyhow::anyhow!("Execution interrupted: {:?}", data))
                    }

                    Update::ExecutionError(data) => {
                        return Err(anyhow::anyhow!("Execution error: {:?}", data))
                    }

                    Update::Progress(_) => {}
                    Update::Status { .. } => {}
                },
                Err(e) => return Err(anyhow::anyhow!("Error occurred: {:?}", e)),
            }
        }

        Ok(vec![])
    }
}
