use ::ichika::status::IntoStatus;

struct _step_0;
impl ::ichika::node::ThreadNode for _step_0 {
    type Request = String;
    type Response = usize;
    fn run(req: Self::Request) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
        { Ok(req.len()) }.into_status()
    }
}
impl ::ichika::node::ThreadNodeEnum for _step_0 {
    fn id() -> &'static str {
        stringify!(_step_0)
    }
}

struct _step_1;
impl ::ichika::node::ThreadNode for _step_1 {
    type Request = usize;
    type Response = String;
    fn run(req: Self::Request) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
        { Ok(req.to_string()) }.into_status()
    }
}
impl ::ichika::node::ThreadNodeEnum for _step_1 {
    fn id() -> &'static str {
        stringify!(_step_1)
    }
}

fn main() {
    let _ = (_step_0, _step_1);
}
