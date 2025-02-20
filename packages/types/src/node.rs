use anyhow::Error;

use crate::status::IntoStatus;

pub trait ThreadNode {
    type Request: Clone;
    type Response: Clone;

    fn run(req: Self::Request) -> impl IntoStatus<Self::Response, Error>;
}

#[async_trait::async_trait]
pub trait ThreadNodeAsync {
    type Request: Clone;
    type Response: Clone;

    async fn run(req: Self::Request) -> impl IntoStatus<Self::Response, Error>;
}

pub trait ThreadNodeEnum {
    fn id() -> &'static str;
}
