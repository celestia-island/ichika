use anyhow::Error;

use crate::status::IntoStatus;

pub trait ThreadNode {
    type Request: Clone;
    type Response: Clone;

    fn run(req: Self::Request) -> impl IntoStatus<Self::Response, Error>;
}

pub trait ThreadNodeEnum {
    fn id() -> &'static str;
}
