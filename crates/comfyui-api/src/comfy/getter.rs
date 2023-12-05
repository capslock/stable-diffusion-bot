use std::collections::HashSet;

use anyhow::Context;

use crate::models::*;
use crate::{comfy::visitor::FindNode, comfy::Visitor};

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
