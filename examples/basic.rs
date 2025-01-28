use anyhow::Result;

use ichika::{create_node, ThreadNode, ThreadPod};

fn main() -> Result<()> {
    create_node!(PipeNode |req: String| -> usize {
        req.len()
    });

    let mut node = PipeNode::default();
    for i in 0..20 {
        node.send(format!("Hello, World! {}", i))?;
        let response = node.recv()?;
        println!("{} -> {}", i, response);
    }

    Ok(())
}
