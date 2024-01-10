use std::collections::HashSet;

use anyhow::{anyhow, Context};

use crate::models::*;
use crate::{comfy::visitor::FindNode, comfy::Visitor};

use super::accessors;

/// A trait for getting values from nodes.
pub trait Getter<T, N>
where
    N: Node + 'static,
    Self: Default,
{
    /// Uses a heuristic to find a `Node` and get the value from it.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// A reference to the value on success, or an error if the node could not be found.
    fn get<'a>(&self, prompt: &'a Prompt) -> anyhow::Result<&'a T> {
        let node = if let Some(node) = Self::guess_node(prompt, None) {
            node
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.get_value(node)
    }

    /// Uses a heuristic to find a `Node` and get the value from it.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// A mutable reference to the value on success, or an error if the node could not be found.
    fn get_mut<'a>(&self, prompt: &'a mut Prompt) -> anyhow::Result<&'a mut T> {
        let node = if let Some(node) = Self::guess_node_mut(prompt, None) {
            node
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.get_value_mut(node)
    }

    /// Finds a `Node` leading into the given `output_node` and gets the value from it.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// A reference to the value on success, or an error if the node could not be found.
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

    /// Finds a `Node` leading into the given `output_node` and gets the value from it.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// A mutable reference to the value on success, or an error if the node could not be found.
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

    /// Gets a value from the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A reference to a `Prompt`.
    /// * `node` - The id of the node to set the value on.
    ///
    /// # Returns
    ///
    /// A reference to the value on success, or an error if the node could not be found.
    fn get_node<'a>(&self, prompt: &'a Prompt, node: &str) -> anyhow::Result<&'a T> {
        let node = prompt.get_node_by_id(node).unwrap();
        self.get_value(node)
    }

    /// Gets a value from the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    /// * `node` - The id of the node to set the value on.
    ///
    /// # Returns
    ///
    /// A mutable reference to the value on success, or an error if the node could not be found.
    fn get_node_mut<'a>(&self, prompt: &'a mut Prompt, node: &str) -> anyhow::Result<&'a mut T> {
        let node = prompt.get_node_by_id_mut(node).unwrap();
        self.get_value_mut(node)
    }

    /// Gets a value from the given `Node`.
    ///
    /// # Inputs
    ///
    /// * `node` - A reference to a `Node`.
    ///
    /// # Returns
    ///
    /// A reference to the value on success, or an error if the node could not be found.
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a T>;

    /// Gets a value from the given `Node`.
    ///
    /// # Inputs
    ///
    /// * `node` - A mutable reference to a `Node`.
    ///
    /// # Returns
    ///
    /// A mutable reference to the value on success, or an error if the node could not be found.
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

    /// Uses a heuristic to find a `Node`.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A reference to a `Prompt`.
    /// * `output_node` - The id of the node to search from.
    ///
    /// # Returns
    ///
    /// A reference to the node on success, or `None` if the node could not be found.
    fn guess_node<'a>(prompt: &'a Prompt, output_node: Option<&str>) -> Option<&'a dyn Node> {
        if let Some(node) = Self::find_node(prompt, output_node) {
            prompt.get_node_by_id(&node)
        } else if let Some((_, node)) = prompt.get_nodes_by_type::<N>().next() {
            Some(node)
        } else {
            None
        }
    }

    /// Uses a heuristic to find a `Node`.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    /// * `output_node` - The id of the node to search from.
    ///
    /// # Returns
    ///
    /// A mutable reference to the node on success, or `None` if the node could not be found.
    fn guess_node_mut<'a>(
        prompt: &'a mut Prompt,
        output_node: Option<&str>,
    ) -> Option<&'a mut dyn Node> {
        if let Some(node) = Self::find_node(prompt, output_node) {
            prompt.get_node_by_id_mut(&node)
        } else if let Some((_, node)) = prompt.get_nodes_by_type_mut::<N>().next() {
            Some(node)
        } else {
            None
        }
    }
}

/// Extension methods for `Prompt` to get nodes.
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
    fn get_typed_node(&self, node: &str) -> anyhow::Result<&N>;

    /// Gets a mutable reference to the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `node` - The id of the node to get the value from.
    ///
    /// # Returns
    ///
    /// A `Result` containing the mutable reference on success, or an error if the node could not be found.
    fn get_typed_node_mut(&mut self, node: &str) -> anyhow::Result<&mut N>;
}

impl<N: Node + 'static> GetExt<N> for Prompt {
    fn get_typed_node(&self, node: &str) -> anyhow::Result<&N> {
        let node = self.get_node_by_id(node).context("failed to get node")?;
        as_node::<N>(node).context("Failed to cast node")
    }

    fn get_typed_node_mut(&mut self, node: &str) -> anyhow::Result<&mut N> {
        let node = self
            .get_node_by_id_mut(node)
            .context("failed to get node")?;
        as_node_mut::<N>(node).context("Failed to cast node")
    }
}

trait GetterExt<T, N>
where
    N: Node + 'static,
{
    fn get<G>(&self) -> anyhow::Result<&T>
    where
        G: Getter<T, N>;

    fn get_mut<G>(&mut self) -> anyhow::Result<&mut T>
    where
        G: Getter<T, N>;

    fn get_from<G>(&self, output_node: &str) -> anyhow::Result<&T>
    where
        G: Getter<T, N>;

    fn get_from_mut<G>(&mut self, output_node: &str) -> anyhow::Result<&mut T>
    where
        G: Getter<T, N>;

    fn get_node<G>(&self, node: &str) -> anyhow::Result<&T>
    where
        G: Getter<T, N>;

    fn get_node_mut<G>(&mut self, node: &str) -> anyhow::Result<&mut T>
    where
        G: Getter<T, N>;
}

impl<T, N: Node + 'static> GetterExt<T, N> for Prompt {
    fn get<G>(&self) -> anyhow::Result<&T>
    where
        G: Getter<T, N>,
    {
        G::default().get(self).context("Failed to get value")
    }

    fn get_mut<G>(&mut self) -> anyhow::Result<&mut T>
    where
        G: Getter<T, N>,
    {
        G::default().get_mut(self).context("Failed to get value")
    }

    fn get_from<G>(&self, output_node: &str) -> anyhow::Result<&T>
    where
        G: Getter<T, N>,
    {
        G::default()
            .get_from(self, output_node)
            .context("Failed to get value")
    }

    fn get_from_mut<G>(&mut self, output_node: &str) -> anyhow::Result<&mut T>
    where
        G: Getter<T, N>,
    {
        G::default()
            .get_from_mut(self, output_node)
            .context("Failed to get value")
    }

    fn get_node<G>(&self, node: &str) -> anyhow::Result<&T>
    where
        G: Getter<T, N>,
    {
        G::default()
            .get_node(self, node)
            .context("Failed to get value")
    }

    fn get_node_mut<G>(&mut self, node: &str) -> anyhow::Result<&mut T>
    where
        G: Getter<T, N>,
    {
        G::default()
            .get_node_mut(self, node)
            .context("Failed to get value")
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

#[allow(dead_code)]
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

macro_rules! create_getter {
    ($ValueType:ty, $NodeType:ty, $AccessorType:ty, $field_name:ident) => {
        impl Getter<$ValueType, $NodeType> for $AccessorType {
            fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a $ValueType> {
                as_node::<$NodeType>(node)
                    .context(concat!("Failed to cast node to ", stringify!($NodeType)))?
                    .$field_name
                    .value()
                    .context(concat!(
                        "Failed to get ",
                        stringify!($getter_name),
                        " value"
                    ))
            }

            fn get_value_mut<'a>(
                &self,
                node: &'a mut dyn Node,
            ) -> anyhow::Result<&'a mut $ValueType> {
                as_node_mut::<$NodeType>(node)
                    .context(concat!("Failed to cast node to ", stringify!($NodeType)))?
                    .$field_name
                    .value_mut()
                    .context(concat!(
                        "Failed to get ",
                        stringify!($getter_name),
                        " value"
                    ))
            }
        }
    };
}

macro_rules! create_ext_trait {
    ($ValueType:ty, $AccessorType:ty, $getter_name:ident, $getter_name_mut:ident, $TraitName:ident) => {
        /// Trait to get references to values from a `Prompt`.
        pub trait $TraitName {
            /// Get a reference to the value.
            ///
            /// # Returns
            ///
            /// A `Result` containing the reference on success, or an error if the node could not be found.
            fn $getter_name(&self) -> anyhow::Result<&$ValueType>;

            /// Get a mutable reference to the value.
            ///
            /// # Returns
            ///
            /// A `Result` containing the mutable reference on success, or an error if the node could not be found.
            fn $getter_name_mut(&mut self) -> anyhow::Result<&mut $ValueType>;
        }

        impl $TraitName for Prompt {
            fn $getter_name(&self) -> anyhow::Result<&$ValueType> {
                self.get::<$AccessorType>()
            }

            fn $getter_name_mut(&mut self) -> anyhow::Result<&mut $ValueType> {
                self.get_mut::<$AccessorType>()
            }
        }
    };
}

impl Getter<String, CLIPTextEncode> for accessors::Prompt {
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
            if let Ok(node) = prompt.get_typed_node(&node) as anyhow::Result<&KSampler> {
                return Some(node.positive.node_id.clone());
            }
        }
        if let Some(node) = find_node::<SamplerCustom>(prompt, output_node) {
            if let Ok(node) = prompt.get_typed_node(&node) as anyhow::Result<&SamplerCustom> {
                return Some(node.positive.node_id.clone());
            }
        }
        None
    }
}

create_ext_trait!(String, accessors::Prompt, prompt, prompt_mut, PromptExt);

impl Getter<String, CLIPTextEncode> for accessors::NegativePrompt {
    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a String> {
        accessors::Prompt.get_value(node)
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut String> {
        accessors::Prompt.get_value_mut(node)
    }

    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        if let Some(node) = find_node::<KSampler>(prompt, output_node) {
            if let Ok(node) = prompt.get_typed_node(&node) as anyhow::Result<&KSampler> {
                return Some(node.negative.node_id.clone());
            }
        }
        if let Some(node) = find_node::<SamplerCustom>(prompt, output_node) {
            if let Ok(node) = prompt.get_typed_node(&node) as anyhow::Result<&SamplerCustom> {
                return Some(node.negative.node_id.clone());
            }
        }
        None
    }
}

create_ext_trait!(
    String,
    accessors::NegativePrompt,
    negative_prompt,
    negative_prompt_mut,
    NegativePromptExt
);

create_getter!(String, CheckpointLoaderSimple, accessors::Model, ckpt_name);
create_ext_trait!(String, accessors::Model, ckpt_name, ckpt_name_mut, ModelExt);

create_getter!(u32, EmptyLatentImage, accessors::Width, width);
create_ext_trait!(u32, accessors::Width, width, width_mut, WidthExt);

create_getter!(u32, EmptyLatentImage, accessors::Height, height);
create_ext_trait!(u32, accessors::Height, height, height_mut, HeightExt);

create_getter!(i64, KSampler, accessors::SeedT<KSampler>, seed);
create_getter!(
    i64,
    SamplerCustom,
    accessors::SeedT<SamplerCustom>,
    noise_seed
);

create_ext_trait!(i64, accessors::Seed, seed, seed_mut, SeedExt);

create_getter!(u32, KSampler, accessors::StepsT<KSampler>, steps);
create_getter!(
    u32,
    SDTurboScheduler,
    accessors::StepsT<SDTurboScheduler>,
    steps
);

create_ext_trait!(u32, accessors::Steps, steps, steps_mut, StepsExt);

impl<S1, S2, T, N1, N2> Getter<T, N1> for accessors::Delegating<S1, S2, T, N1, N2>
where
    S1: Getter<T, N1>,
    S2: Getter<T, N2>,
    N1: Node + 'static,
    N2: Node + 'static,
    T: Clone + Default,
{
    fn get<'a>(&self, prompt: &'a Prompt) -> anyhow::Result<&'a T> {
        S1::default()
            .get(prompt)
            .or_else(|_| S2::default().get(prompt).context("Failed to set value"))
    }

    fn get_mut<'a>(&self, prompt: &'a mut Prompt) -> anyhow::Result<&'a mut T> {
        let s1 = S1::default();
        if s1.get(prompt).is_ok() {
            return s1.get_mut(prompt);
        }
        S2::default().get_mut(prompt).context("Failed to set value")
    }

    fn get_from<'a>(&self, prompt: &'a Prompt, output_node: &str) -> anyhow::Result<&'a T> {
        S1::default().get_from(prompt, output_node).or_else(|_| {
            S2::default()
                .get_from(prompt, output_node)
                .context("Failed to set value")
        })
    }

    fn get_from_mut<'a>(
        &self,
        prompt: &'a mut Prompt,
        output_node: &str,
    ) -> anyhow::Result<&'a mut T> {
        let s1 = S1::default();
        if s1.get_from(prompt, output_node).is_ok() {
            return s1.get_from_mut(prompt, output_node);
        }
        S2::default()
            .get_from_mut(prompt, output_node)
            .context("Failed to set value")
    }

    fn get_node<'a>(&self, prompt: &'a Prompt, node: &str) -> anyhow::Result<&'a T> {
        S1::default().get_node(prompt, node).or_else(|_| {
            S2::default()
                .get_node(prompt, node)
                .context("Failed to set value")
        })
    }

    fn get_node_mut<'a>(&self, prompt: &'a mut Prompt, node: &str) -> anyhow::Result<&'a mut T> {
        let s1 = S1::default();
        if s1.get_node(prompt, node).is_ok() {
            return s1.get_node_mut(prompt, node);
        }
        S2::default()
            .get_node_mut(prompt, node)
            .context("Failed to set value")
    }

    fn get_value<'a>(&self, node: &'a dyn Node) -> anyhow::Result<&'a T> {
        S1::default()
            .get_value(node)
            .or_else(|_| S2::default().get_value(node).context("Failed to set value"))
    }

    fn get_value_mut<'a>(&self, node: &'a mut dyn Node) -> anyhow::Result<&'a mut T> {
        let s1 = S1::default();
        if s1.get_value(node).is_ok() {
            return s1.get_value_mut(node);
        }
        S2::default()
            .get_value_mut(node)
            .context("Failed to set value")
    }

    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        find_node::<N1>(prompt, output_node).or_else(|| find_node::<N2>(prompt, output_node))
    }
}

create_getter!(f32, KSampler, accessors::CfgT<KSampler>, cfg);
create_getter!(f32, SamplerCustom, accessors::CfgT<SamplerCustom>, cfg);
create_ext_trait!(f32, accessors::Cfg, cfg, cfg_mut, CfgExt);

create_getter!(f32, KSampler, accessors::Denoise, denoise);
create_ext_trait!(f32, accessors::Denoise, denoise, denoise_mut, DenoiseExt);

create_getter!(
    String,
    KSampler,
    accessors::SamplerT<KSampler>,
    sampler_name
);
create_getter!(
    String,
    KSamplerSelect,
    accessors::SamplerT<KSamplerSelect>,
    sampler_name
);
create_ext_trait!(
    String,
    accessors::Sampler,
    sampler_name,
    sampler_name_mut,
    SamplerExt
);

create_getter!(u32, EmptyLatentImage, accessors::BatchSize, batch_size);
create_ext_trait!(
    u32,
    accessors::BatchSize,
    batch_size,
    batch_size_mut,
    BatchSizeExt
);

create_getter!(String, LoadImage, accessors::LoadImage, image);
create_ext_trait!(String, accessors::LoadImage, image, image_mut, LoadImageExt);
