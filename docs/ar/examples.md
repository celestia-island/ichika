# الأمثلة

تحتوي هذه الصفحة على أمثلة عملية توضح ميزات Ichika المختلفة.

## جدول المحتويات

- [خط الأنابيب المتزامن الأساسي](#خط-الأنابيب-المتزامن-الأساسي)
- [خط الأنابيب غير المتزامن الأساسي](#خط-الأنابيب-غير-المتزامن-الأساسي)
- [معالجة الأخطاء](#معالجة-الأخطاء)
- [إيقاف التشغيل بأناقة](#إيقاف-التشغيل-بأناقة)
- [مراقبة استخدام مؤشرات الترابط](#مراقبة-استخدام-مؤشرات-الترابط)
- [خط الأنابيب مع حمولة tuple](#خط-الأنابيب-مع-حمولة-tuple)

## خط الأنابيب المتزامن الأساسي

مثال أدنى يوضح خط أنابيب متزامن بسيط من مرحلتين:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("تحويل '{}' إلى طول", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("تحويل الطول {} مرة أخرى إلى سلسلة", req);
            Ok(req.to_string())
        }
    ]?;

    let inputs = vec!["hello", "world", "ichika"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    loop {
        match pool.recv()? {
            Some(output) => log::info!("تم الاستلام: {}", output),
            None => break,
        }
    }

    Ok(())
}
```

## خط الأنابيب غير المتزامن الأساسي

مثال باستخدام مراحل غير متزامنة مع tokio:

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("المرحلة 1: {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("المرحلة 2: معالجة {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("النتيجة: {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## معالجة الأخطاء

توضيح انتشار الأخطاء عبر خط الأنابيب:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("parse")]
        |req: String| -> anyhow::Result<i32> {
            log::info!("تحليل: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("معالجة: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("النتيجة: {}", n),
                Err(e) => format!("الخطأ: {}", e),
            }
        }
    ]?;

    let inputs = vec!["42", "100", "invalid", "200"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    std::thread::sleep(std::time::Duration::from_millis(100));

    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## إيقاف التشغيل بأناقة

توضيح التنظيف المناسب عند إسقاط خط الأنابيب:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("معالجة: {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // إرسال العمل
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // إعطاء وقت للمعالجة
        std::thread::sleep(Duration::from_millis(200));

        // سيتم إسقاط المجموعة وإيقاف التشغيل بأناقة
        log::info!("المجموعة تخرج من النطاق...");
    }

    log::info!("تم إيقاف المجموعة بأناقة");

    Ok(())
}
```

## مراقبة استخدام مؤشرات الترابط

تتبع استخدام مؤشرات الترابط وأعداد المهام:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("stage1")]
        |req: String| -> usize {
            std::thread::sleep(Duration::from_millis(100));
            req.len()
        },
        #[name("stage2")]
        |req: usize| -> String {
            req.to_string()
        }
    ]?;

    // إرسال العمل
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // مراقبة التقدم
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "مؤشرات الترابط: {}، Stage1: {}، Stage2: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("جميع المهام مكتملة");

    Ok(())
}
```

## خط الأنابيب مع حمولة tuple

العمل مع حمولات tuple:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}' له طول {}", req.0, req.1)
        }
    ]?;

    pool.send("hello".to_string())?;
    pool.send("world".to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## تشغيل الأمثلة

جميع الأمثلة متاحة في المستودع:

```bash
# تشغيل مثال محدد
cargo run --example basic_sync_chain

# تشغيل مع التسجيل
RUST_LOG=info cargo run --example basic_sync_chain

# تشغيل المثال غير المتزامن
cargo run --example basic_async_chain --features tokio
```

## المزيد من الأمثلة

راجع دليل `examples/` في المستودع للمزيد من الأمثلة الكاملة:

- `basic_sync_chain.rs` — خط الأنابيب المتزامن
- `basic_async_chain.rs` — خط الأنابيب غير المتزامن
- `error_handling.rs` — انتشار الأخطاء
- `graceful_shutdown_drop.rs` — التنظيف عند الإسقاط
- `monitoring_thread_usage.rs` — واجهة برمجة تطبيقات المراقبة
- `tuple_payload_pipeline.rs` — أنواع حمولات معقدة
- `status_exit_demo.rs` — إدارة الحالة والخروج
