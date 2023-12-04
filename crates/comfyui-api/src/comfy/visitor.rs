use crate::models::*;

use super::ImageInfo;

/// Trait for visiting nodes in a ComfyUI graph.
pub trait Visitor {
    /// Visits a node in a ComfyUI graph.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt that contains the graph.
    /// * `node` - The node to visit.
    fn visit(&mut self, prompt: &Prompt, node: &dyn Node) {
        for c in node.connections() {
            if let Some(node) = prompt.get_node_by_id(c) {
                self.visit(prompt, node);
            }
        }
    }
}

impl Visitor for ImageInfo {
    fn visit(&mut self, prompt: &Prompt, node: &dyn Node) {
        if let Some(node) = as_node::<CheckpointLoaderSimple>(node) {
            self.model = node.ckpt_name.value().cloned();
        } else if let Some(node) = as_node::<ImageOnlyCheckpointLoader>(node) {
            self.model = node.ckpt_name.value().cloned();
        } else if let Some(node) = as_node::<EmptyLatentImage>(node) {
            self.width = node.width.value().cloned();
            self.height = node.height.value().cloned();
        } else if let Some(node) = as_node::<KSampler>(node) {
            self.seed = node.seed.value().cloned();
        } else if let Some(node) = as_node::<SamplerCustom>(node) {
            self.seed = node.noise_seed.value().cloned();
        } else if let Some(node) = as_node::<CLIPTextEncode>(node) {
            if self.prompt.is_none() {
                self.prompt = node.text.value().cloned();
            } else if self.negative_prompt.is_none() {
                self.negative_prompt = node.text.value().cloned();
            }
        }
        for c in node.connections() {
            if let Some(node) = prompt.get_node_by_id(c) {
                self.visit(prompt, node);
            }
        }
    }
}

pub(crate) struct FindNode<T: Node + 'static> {
    pub(crate) visiting: String,
    pub(crate) found: Option<String>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Node + 'static> FindNode<T> {
    pub(crate) fn new(start: String) -> Self {
        Self {
            visiting: start,
            found: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Node + 'static> Visitor for FindNode<T> {
    fn visit(&mut self, prompt: &Prompt, node: &dyn Node) {
        if let Some(_node) = as_node::<T>(node) {
            self.found = Some(self.visiting.clone());
        }
        for c in node.connections() {
            if let Some(node) = prompt.get_node_by_id(c) {
                self.visiting = c.to_string();
                self.visit(prompt, node);
            }
        }
    }
}
