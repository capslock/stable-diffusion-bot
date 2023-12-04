use anyhow::{anyhow, Context};

use crate::{comfy::visitor::FindNode, comfy::Visitor, models::*};

pub trait Setter<T, N>
where
    N: Node + 'static,
{
    fn set(&self, prompt: &mut Prompt, value: T) -> anyhow::Result<()> {
        let node = if let Some(node) = guess_node::<N>(prompt, None) {
            node
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.set_value(node, value)
    }

    fn set_from(&self, prompt: &mut Prompt, output_node: &str, value: T) -> anyhow::Result<()> {
        let node = if let Some(node) = find_node::<N>(prompt, Some(output_node)) {
            prompt
                .get_node_by_id_mut(&node)
                .context("Failed to find node")?
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.set_value(node, value)
    }

    fn set_node(&self, prompt: &mut Prompt, node: &str, value: T) -> anyhow::Result<()> {
        let node = prompt.get_node_by_id_mut(node).unwrap();
        self.set_value(node, value)
    }

    fn set_value(&self, node: &mut dyn Node, value: T) -> anyhow::Result<()>;
}

fn start_node(prompt: &Prompt) -> Option<String> {
    if let Some((node, _)) = prompt.get_nodes_by_type::<PreviewImage>().next() {
        Some(node.to_string())
    } else if let Some((node, _)) = prompt.get_nodes_by_type::<SaveImage>().next() {
        Some(node.to_string())
    } else {
        None
    }
}

fn find_node<T: Node + 'static>(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
    let output_node = if let Some(node) = output_node {
        node.to_string()
    } else {
        start_node(prompt)?
    };
    let mut find_node = FindNode::<T>::new(output_node.clone());
    find_node.visit(prompt, prompt.get_node_by_id(&output_node).unwrap());
    find_node.found
}

fn guess_node<'a, T: Node + 'static>(
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

pub(crate) struct PromptSetter {}

impl Setter<String, CLIPTextEncode> for PromptSetter {
    fn set_value(&self, node: &mut dyn Node, value: String) -> anyhow::Result<()> {
        *as_node_mut::<CLIPTextEncode>(node)
            .context("Failed to cast node")?
            .text
            .value_mut()
            .context("Failed to get text value")? = value;
        Ok(())
    }
}

pub(crate) struct ModelSetter {}

impl Setter<String, CheckpointLoaderSimple> for ModelSetter {
    fn set_value(&self, node: &mut dyn Node, value: String) -> anyhow::Result<()> {
        *as_node_mut::<CheckpointLoaderSimple>(node)
            .context("Failed to cast node")?
            .ckpt_name
            .value_mut()
            .context("Failed to get ckpt_name value")? = value;
        Ok(())
    }
}

pub(crate) struct SizeSetter {}

impl Setter<(u32, u32), EmptyLatentImage> for SizeSetter {
    fn set_value(&self, node: &mut dyn Node, value: (u32, u32)) -> anyhow::Result<()> {
        if value.0 > 0 {
            *as_node_mut::<EmptyLatentImage>(node)
                .context("Failed to cast node")?
                .width
                .value_mut()
                .context("Failed to get width value")? = value.0;
        }
        if value.1 > 0 {
            *as_node_mut::<EmptyLatentImage>(node)
                .context("Failed to cast node")?
                .height
                .value_mut()
                .context("Failed to get height value")? = value.1;
        }
        Ok(())
    }
}

pub(crate) struct SeedSetter {}

impl Setter<i64, KSampler> for SeedSetter {
    fn set_value(&self, node: &mut dyn Node, value: i64) -> anyhow::Result<()> {
        *as_node_mut::<KSampler>(node)
            .context("Failed to cast node")?
            .seed
            .value_mut()
            .context("Failed to get seed value")? = value;
        Ok(())
    }
}
