pub mod pipeline;

use anyhow::Result;

use pipeline::Pipeline;

/// Create a new thread pool.
pub fn create_pool<I, O, F>(f: F) -> Pipeline<I, O, F>
where
    F: Fn(I) -> Result<O> + 'static,
{
    Pipeline::new(f)
}
