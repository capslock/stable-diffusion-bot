use std::collections::HashSet;

use anyhow::{anyhow, Context};

use crate::models::*;
use crate::{comfy::visitor::FindNode, comfy::Visitor};

use super::{
    DelegatingSetter, HeightSetter, ModelSetter, NegativePromptSetter, PromptSetter, SeedSetterT,
    WidthSetter,
};

/// A trait for setting values on nodes.
pub trait Getter<T, N>
where
    N: Node + 'static,
    Self: From<T>,
{
    /// Uses a heuristic to find a `Node` and set the value on it.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn get<'a>(&self, prompt: &'a Prompt) -> anyhow::Result<&'a T> {
        let node = if let Some(node) = guess_node::<N>(prompt, None) {
            node
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.get_value(node)
    }

    fn get_mut<'a>(&self, prompt: &'a mut Prompt) -> anyhow::Result<&'a mut T> {
        let node = if let Some(node) = guess_node_mut::<N>(prompt, None) {
            node
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.get_value_mut(node)
    }

    /// Finds a `Node` leading into the given `output_node` and sets the value on it.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn get_from<'a>(&self, prompt: &'a Prompt, output_node: &str) -> anyhow::Result<&'a T> {
        let node = if let Some(node) = Self::find_node(prompt, Some(output_node)) {
            prompt
                .get_node_by_id(&node)
                .context("Failed to find node")?
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.get_value(node)
    }

    fn get_from_mut<'a>(
        &self,
        prompt: &'a mut Prompt,
        output_node: &str,
    ) -> anyhow::Result<&'a mut T> {
        let node = if let Some(node) = Self::find_node(prompt, Some(output_node)) {
            prompt
                .get_node_by_id_mut(&node)
                .context("Failed to find node")?
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.get_value_mut(node)
    }

    /// Sets the value on the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    /// * `node` - The id of the node to set the value on.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn get_node<'a>(&self, prompt: &'a Prompt, node: &str) -> anyhow::Result<&'a T> {
        let node = prompt.get_node_by_id(node).unwrap();
        self.get_value(node)
    }

    fn get_node_mut<'a>(&self, prompt: &'a mut Prompt, node: &str) -> anyhow::Result<&'a mut T> {
        let node = prompt.get_node_by_id_mut(node).unwrap();
        self.get_value_mut(node)
    }

    /// Sets the value on the given `Node`.
    ///
    /// # Inputs
    ///
    /// * `node` - A mutable reference to a `Node`.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a T>;

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut T>;

    /// Finds a `Node` leading into the given `output_node`.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// The id of the node on success, or `None` if the node could not be found.
    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        find_node::<N>(prompt, output_node)
    }
}

/// Extension methods for `Prompt` to get values from nodes.
pub trait GetExt<N>
where
    N: Node + 'static,
{
    /// Gets a reference to the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `node` - The id of the node to get the value from.
    ///
    /// # Returns
    ///
    /// A `Result` containing the reference on success, or an error if the node could not be found.
    fn get_node(&self, node: &str) -> anyhow::Result<&N>;

    /// Gets a mutable reference to the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `node` - The id of the node to get the value from.
    ///
    /// # Returns
    ///
    /// A `Result` containing the mutable reference on success, or an error if the node could not be found.
    fn get_node_mut(&mut self, node: &str) -> anyhow::Result<&mut N>;
}

impl<N: Node + 'static> GetExt<N> for Prompt {
    fn get_node(&self, node: &str) -> anyhow::Result<&N> {
        let node = self.get_node_by_id(node).context("failed to get node")?;
        as_node::<N>(node).context("Failed to cast node")
    }

    fn get_node_mut(&mut self, node: &str) -> anyhow::Result<&mut N> {
        let node = self
            .get_node_by_id_mut(node)
            .context("failed to get node")?;
        as_node_mut::<N>(node).context("Failed to cast node")
    }
}

pub(crate) fn find_node<T: Node + 'static>(
    prompt: &Prompt,
    output_node: Option<&str>,
) -> Option<String> {
    let output_node = if let Some(node) = output_node {
        node.to_string()
    } else {
        find_output_node(prompt)?
    };
    let mut find_node = FindNode::<T>::new(output_node.clone());
    find_node.visit(prompt, prompt.get_node_by_id(&output_node).unwrap());
    find_node.found
}

pub(crate) fn guess_node<'a, T: Node + 'static>(
    prompt: &'a Prompt,
    output_node: Option<&str>,
) -> Option<&'a dyn Node> {
    if let Some(node) = find_node::<T>(prompt, output_node) {
        prompt.get_node_by_id(&node)
    } else if let Some((_, node)) = prompt.get_nodes_by_type::<T>().next() {
        Some(node)
    } else {
        None
    }
}

pub(crate) fn guess_node_mut<'a, T: Node + 'static>(
    prompt: &'a mut Prompt,
    output_node: Option<&str>,
) -> Option<&'a mut dyn Node> {
    if let Some(node) = find_node::<T>(prompt, output_node) {
        prompt.get_node_by_id_mut(&node)
    } else if let Some((_, node)) = prompt.get_nodes_by_type_mut::<T>().next() {
        Some(node)
    } else {
        None
    }
}

pub(crate) fn find_output_node(prompt: &Prompt) -> Option<String> {
    let nodes: HashSet<String> = prompt.workflow.keys().cloned().collect();
    prompt
        .workflow
        .iter()
        .fold(nodes, |mut nodes, (key, value)| {
            let mut has_input = false;
            let connections = match value {
                NodeOrUnknown::Node(node) => node.connections(),
                NodeOrUnknown::GenericNode(node) => node.connections(),
            };
            for c in connections {
                has_input = true;
                nodes.remove(c);
            }
            if !has_input {
                nodes.remove(key);
            }
            nodes
        })
        .into_iter()
        .next()
}

impl Getter<String, CLIPTextEncode> for PromptSetter {
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a String> {
        as_node::<CLIPTextEncode>(node)
            .context("Failed to cast node")?
            .text
            .value()
            .context("Failed to get text value")
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut String> {
        as_node_mut::<CLIPTextEncode>(node)
            .context("Failed to cast node")?
            .text
            .value_mut()
            .context("Failed to get text value")
    }

    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        if let Some(node) = find_node::<KSampler>(prompt, output_node) {
            if let Ok(node) = prompt.get_node(&node) as anyhow::Result<&KSampler> {
                return Some(node.positive.node_id.clone());
            }
        }
        if let Some(node) = find_node::<SamplerCustom>(prompt, output_node) {
            if let Ok(node) = prompt.get_node(&node) as anyhow::Result<&SamplerCustom> {
                return Some(node.positive.node_id.clone());
            }
        }
        None
    }
}

impl Getter<String, CLIPTextEncode> for NegativePromptSetter {
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a String> {
        PromptSetter::from(self).get_value(node)
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut String> {
        PromptSetter::from(self).get_value_mut(node)
    }

    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        if let Some(node) = find_node::<KSampler>(prompt, output_node) {
            if let Ok(node) = prompt.get_node(&node) as anyhow::Result<&KSampler> {
                return Some(node.negative.node_id.clone());
            }
        }
        if let Some(node) = find_node::<SamplerCustom>(prompt, output_node) {
            if let Ok(node) = prompt.get_node(&node) as anyhow::Result<&SamplerCustom> {
                return Some(node.negative.node_id.clone());
            }
        }
        None
    }
}

impl Getter<String, CheckpointLoaderSimple> for ModelSetter {
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a String> {
        as_node::<CheckpointLoaderSimple>(node)
            .context("Failed to cast node")?
            .ckpt_name
            .value()
            .context("Failed to get ckpt_name value")
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut String> {
        as_node_mut::<CheckpointLoaderSimple>(node)
            .context("Failed to cast node")?
            .ckpt_name
            .value_mut()
            .context("Failed to get ckpt_name value")
    }
}

impl Getter<u32, EmptyLatentImage> for WidthSetter {
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a u32> {
        as_node::<EmptyLatentImage>(node)
            .context("Failed to cast node")?
            .width
            .value()
            .context("Failed to get width value")
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut u32> {
        as_node_mut::<EmptyLatentImage>(node)
            .context("Failed to cast node")?
            .width
            .value_mut()
            .context("Failed to get width value")
    }
}

impl Getter<u32, EmptyLatentImage> for HeightSetter {
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a u32> {
        as_node::<EmptyLatentImage>(node)
            .context("Failed to cast node")?
            .height
            .value()
            .context("Failed to get height value")
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut u32> {
        as_node_mut::<EmptyLatentImage>(node)
            .context("Failed to cast node")?
            .height
            .value_mut()
            .context("Failed to get height value")
    }
}

impl Getter<i64, KSampler> for SeedSetterT<KSampler> {
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a i64> {
        as_node::<KSampler>(node)
            .context("Failed to cast node")?
            .seed
            .value()
            .context("Failed to get seed value")
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut i64> {
        as_node_mut::<KSampler>(node)
            .context("Failed to cast node")?
            .seed
            .value_mut()
            .context("Failed to get seed value")
    }
}

impl Getter<i64, SamplerCustom> for SeedSetterT<SamplerCustom> {
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a i64> {
        as_node::<SamplerCustom>(node)
            .context("Failed to cast node")?
            .noise_seed
            .value()
            .context("Failed to get seed value")
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut i64> {
        as_node_mut::<SamplerCustom>(node)
            .context("Failed to cast node")?
            .noise_seed
            .value_mut()
            .context("Failed to get seed value")
    }
}

impl<S1, S2, T, N1, N2> Getter<T, N1> for DelegatingSetter<S1, S2, T, N1, N2>
where
    S1: Getter<T, N1>,
    S2: Getter<T, N2>,
    N1: Node + 'static,
    N2: Node + 'static,
    T: Clone,
{
    fn get<'a>(&self, prompt: &'a Prompt) -> anyhow::Result<&'a T> {
        S1::from(self.value.clone()).get(prompt).or_else(|_| {
            S2::from(self.value.clone())
                .get(prompt)
                .context("Failed to set value")
        })
    }

    fn get_mut<'a>(&self, prompt: &'a mut Prompt) -> anyhow::Result<&'a mut T> {
        let s1 = S1::from(self.value.clone());
        if s1.get(prompt).is_ok() {
            return s1.get_mut(prompt);
        }
        S2::from(self.value.clone())
            .get_mut(prompt)
            .context("Failed to set value")
    }

    fn get_from<'a>(&self, prompt: &'a Prompt, output_node: &str) -> anyhow::Result<&'a T> {
        S1::from(self.value.clone())
            .get_from(prompt, output_node)
            .or_else(|_| {
                S2::from(self.value.clone())
                    .get_from(prompt, output_node)
                    .context("Failed to set value")
            })
    }

    fn get_from_mut<'a>(
        &self,
        prompt: &'a mut Prompt,
        output_node: &str,
    ) -> anyhow::Result<&'a mut T> {
        let s1 = S1::from(self.value.clone());
        if s1.get_from(prompt, output_node).is_ok() {
            return s1.get_from_mut(prompt, output_node);
        }
        S2::from(self.value.clone())
            .get_from_mut(prompt, output_node)
            .context("Failed to set value")
    }

    fn get_node<'a>(&self, prompt: &'a Prompt, node: &str) -> anyhow::Result<&'a T> {
        S1::from(self.value.clone())
            .get_node(prompt, node)
            .or_else(|_| {
                S2::from(self.value.clone())
                    .get_node(prompt, node)
                    .context("Failed to set value")
            })
    }

    fn get_node_mut<'a>(&self, prompt: &'a mut Prompt, node: &str) -> anyhow::Result<&'a mut T> {
        let s1 = S1::from(self.value.clone());
        if s1.get_node(prompt, node).is_ok() {
            return s1.get_node_mut(prompt, node);
        }
        S2::from(self.value.clone())
            .get_node_mut(prompt, node)
            .context("Failed to set value")
    }

    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a T> {
        S1::from(self.value.clone()).get_value(node).or_else(|_| {
            S2::from(self.value.clone())
                .get_value(node)
                .context("Failed to set value")
        })
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut T> {
        let s1 = S1::from(self.value.clone());
        if s1.get_value(node).is_ok() {
            return s1.get_value_mut(node);
        }
        S2::from(self.value.clone())
            .get_value_mut(node)
            .context("Failed to set value")
    }

    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        find_node::<N1>(prompt, output_node).or_else(|| find_node::<N2>(prompt, output_node))
    }
}
