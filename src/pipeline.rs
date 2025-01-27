use anyhow::Result;
use std::marker::PhantomData;

pub struct Pipeline<I, O, F> {
    processor: F,
    _phantom: PhantomData<(I, O)>,
}

impl<I, O, F> Pipeline<I, O, F>
where
    F: FnOnce(I) -> Result<O> + 'static,
{
    /// Create a new pipeline stage.
    pub fn new(processor: F) -> Self {
        Self {
            processor,
            _phantom: PhantomData,
        }
    }

    /// Create a new pipeline stage.
    /// G is the new closure type, O2 is the new output type.
    pub fn pipe<O2, G>(self, next: G) -> Pipeline<I, O2, impl FnOnce(I) -> Result<O2>>
    where
        G: Fn(O) -> Result<O2> + 'static,
    {
        Pipeline::new(move |input: I| {
            let res = (self.processor)(input)?;
            next(res)
        })
    }

    /// Execute the entire pipeline chain.
    pub fn run(self, input: I) -> Result<O> {
        (self.processor)(input)
    }
}
