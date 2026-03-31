use anyhow::Result;
use ichika::pool::ThreadPool;

#[test]
fn test_single_step() -> Result<()> {
    let pool = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) }
    ];

    pool.send("hello".to_string()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    loop {
        if let Some(res) = pool.recv().unwrap() {
            assert_eq!(res, 5);
            break;
        }
    }

    Ok(())
}
