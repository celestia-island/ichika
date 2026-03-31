use anyhow::Error;

use crate::status::Status;

pub trait ThreadNode {
    type Request: Clone;
    type Response: Clone;

    fn run(req: Self::Request) -> Status<Self::Response, Error>;
}

pub trait ThreadNodeEnum {
    fn id() -> &'static str;
}
