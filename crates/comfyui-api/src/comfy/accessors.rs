use crate::models::*;

use super::Getter;

/// A `Setter` for setting the prompt text.
#[derive(Clone, Debug, Default)]
pub(crate) struct Prompt;

/// A `Setter` for setting the negative prompt text.
#[derive(Clone, Debug, Default)]
pub(crate) struct NegativePrompt;

/// A `Setter` for setting the model.
#[derive(Clone, Debug, Default)]
pub(crate) struct Model;

/// A `Setter` for setting the image size.
#[derive(Clone, Debug, Default)]
pub(crate) struct Width;

/// A `Setter` for setting the image size.
#[derive(Clone, Debug, Default)]
pub(crate) struct Height;

/// A `Setter` for setting the seed. Generic over the node type.
#[derive(Clone, Debug)]
pub(crate) struct SeedT<N>
where
    N: Node + 'static,
{
    pub _phantom: std::marker::PhantomData<N>,
}

impl<N> Default for SeedT<N>
where
    N: Node + 'static,
{
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A `Setter` for setting the seed.
pub(crate) type Seed =
    Delegating<SeedT<KSampler>, SeedT<SamplerCustom>, i64, KSampler, SamplerCustom>;

#[derive(Clone, Debug)]
pub(crate) struct StepsT<N>
where
    N: Node + 'static,
{
    pub _phantom: std::marker::PhantomData<N>,
}

impl<N> Default for StepsT<N>
where
    N: Node + 'static,
{
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A `Setter` for setting the seed.
pub(crate) type Steps =
    Delegating<StepsT<KSampler>, StepsT<SDTurboScheduler>, u32, KSampler, SDTurboScheduler>;

/// A `Setter` that delegates to two other `Setter`s.
#[derive(Clone, Debug)]
pub(crate) struct Delegating<S1, S2, T, N1, N2>
where
    S1: Getter<T, N1>,
    S2: Getter<T, N2>,
    N1: Node + 'static,
    N2: Node + 'static,
    T: Clone + Default,
{
    _phantom: std::marker::PhantomData<(S1, S2, T, N1, N2)>,
}

impl<S1, S2, T, N1, N2> Default for Delegating<S1, S2, T, N1, N2>
where
    S1: Getter<T, N1>,
    S2: Getter<T, N2>,
    N1: Node + 'static,
    N2: Node + 'static,
    T: Clone + Default,
{
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CfgT<N>
where
    N: Node + 'static,
{
    pub _phantom: std::marker::PhantomData<N>,
}

impl<N> Default for CfgT<N>
where
    N: Node + 'static,
{
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A `Setter` for setting the seed.
pub(crate) type Cfg = Delegating<CfgT<KSampler>, CfgT<SamplerCustom>, f32, KSampler, SamplerCustom>;

#[derive(Clone, Debug, Default)]
pub(crate) struct Denoise;

#[derive(Clone, Debug)]
pub(crate) struct SamplerT<N>
where
    N: Node + 'static,
{
    pub _phantom: std::marker::PhantomData<N>,
}

impl<N> Default for SamplerT<N>
where
    N: Node + 'static,
{
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub(crate) type Sampler =
    Delegating<SamplerT<KSampler>, SamplerT<KSamplerSelect>, String, KSampler, KSamplerSelect>;

#[derive(Clone, Debug, Default)]
pub(crate) struct BatchSize;
