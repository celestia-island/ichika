use anyhow::Result;
use ichika::flume;

#[test]
fn test_channels() -> Result<()> {
    // Test if channels with explicit types work
    let (tx_string, rx_string): (flume::Sender<String>, flume::Receiver<String>) = flume::unbounded();
    let (tx_usize, rx_usize): (flume::Sender<usize>, flume::Receiver<usize>) = flume::unbounded();
    
    tx_string.send("test".to_string())?;
    tx_usize.send(42)?;
    
    assert_eq!(rx_string.recv()?, "test");
    assert_eq!(rx_usize.recv()?, 42);
    
    Ok(())
}
