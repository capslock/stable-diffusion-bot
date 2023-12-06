use anyhow::{anyhow, Context};

use crate::{
    comfy::getter::{guess_node_mut, GetExt, Getter},
    models::*,
};

/// Extension methods for `Prompt` to use Setters to set values on nodes.
pub trait SetterExt<T, N>
where
    N: Node + 'static,
{
    /// Uses a heuristic to find a `Node` and set the value on it.
    ///
    /// # Inputs
    ///
    /// * `value` - The value to set.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set<S>(&mut self, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>;

    /// Finds a `Node` leading into the given `output_node` and sets the value on it.
    ///
    /// # Inputs
    ///
    /// * `output_node` - The id of the node to path from.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_from<S>(&mut self, output_node: &str, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>;

    /// Sets the value on the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `node` - The id of the node to set the value on.
    /// * `value` - The value to set.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_node<S>(&mut self, node: &str, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>;
}

/// Extension methods for `Prompt` to set values on nodes.
pub trait SetExt<N>
where
    N: Node + 'static,
{
    /// Uses a heuristic to find a `Node` and set the value on it.
    ///
    /// # Inputs
    ///
    /// * `f` - A function that takes a mutable reference to the node and returns a `Result`.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_with<F>(&mut self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut N) -> anyhow::Result<()>;

    /// Sets the value on the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `node` - The id of the node to set the value on.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_node_with<F>(&mut self, node: &str, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut N) -> anyhow::Result<()>;
}

impl<T, N: Node + 'static> SetterExt<T, N> for crate::models::Prompt {
    fn set<S>(&mut self, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>,
    {
        S::default().set(self, value)
    }

    fn set_from<S>(&mut self, output_node: &str, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>,
    {
        S::default().set_from(self, output_node, value)
    }

    fn set_node<S>(&mut self, node: &str, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>,
    {
        S::default().set_node(self, node, value)
    }
}

impl<N: Node + 'static> SetExt<N> for crate::models::Prompt {
    fn set_with<F>(&mut self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut N) -> anyhow::Result<()>,
    {
        if let Some(node) = guess_node_mut::<N>(self, None) {
            f(as_node_mut::<N>(node).context("Failed to cast node")?)
        } else {
            Err(anyhow!("Failed to find node"))
        }
    }

    fn set_node_with<F>(&mut self, node: &str, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut N) -> anyhow::Result<()>,
    {
        f(self.get_typed_node_mut(node)?)
    }
}

/// A trait for setting values on nodes.
pub trait Setter<T, N>
where
    N: Node + 'static,
    Self: Getter<T, N>,
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
    fn set(&self, prompt: &mut crate::models::Prompt, value: T) -> anyhow::Result<()> {
        let node = if let Some(node) = guess_node_mut::<N>(prompt, None) {
            node
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.set_value(node, value)
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
    fn set_from(
        &self,
        prompt: &mut crate::models::Prompt,
        output_node: &str,
        value: T,
    ) -> anyhow::Result<()> {
        let node = if let Some(node) = Self::find_node(prompt, Some(output_node)) {
            prompt
                .get_node_by_id_mut(&node)
                .context("Failed to find node")?
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.set_value(node, value)
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
    fn set_node(
        &self,
        prompt: &mut crate::models::Prompt,
        node: &str,
        value: T,
    ) -> anyhow::Result<()> {
        let node = prompt.get_node_by_id_mut(node).unwrap();
        self.set_value(node, value)
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
    fn set_value(&self, node: &mut dyn Node, value: T) -> anyhow::Result<()> {
        *self.get_value_mut(node)? = value;
        Ok(())
    }
}

impl<G, T, N> Setter<T, N> for G
where
    G: Getter<T, N>,
    N: Node + 'static,
{
}
