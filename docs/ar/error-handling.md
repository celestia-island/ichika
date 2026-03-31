# معالجة الأخطاء وإعادة المحاولة

يوفر Ichika معالجة قوية للأخطاء مع دلالات إعادة المحاولة المدمجة للتعامل مع الأعطال المؤقتة.

## انتشار الأخطاء

تتدفق الأخطاء بشكل طبيعي عبر خط الأنابيب باستخدام أنواع `Result`:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
        let n = req?;
        Ok(n * 2)
    },
    |req: anyhow::Result<i32>| -> String {
        match req {
            Ok(n) => format!("النتيجة: {}", n),
            Err(e) => format!("خطأ: {}", e),
        }
    }
]?;
```

### تحويل النوع

عندما تُرجع مرحلة ما `Result`، تتلقى المرحلة التالية هذا `Result`:

```rust
|req: String| -> anyhow::Result<usize> { ... }  # تُرجع Result
|req: anyhow::Result<usize>| -> usize {         # تتلقى Result
    req.unwrap()
}
```

## دلالات إعادة المحاولة

يوفر Ichika إعادة محاولة تلقائية للعمليات التي قد تفشل بشكل مؤقت.

### إعادة المحاولة الأساسية

استخدم الدالة `retry` لإعادة محاولة عملية:

```rust
use ichika::retry;

let result = retry(|| {
    // عملية قد تفشل
    Ok::<_, anyhow::Error>(42)
})?;
```

### إعادة المحاولة مع سياسة

تحكم في سلوك إعادة المحاولة باستخدام `RetryPolicy`:

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // عملية مع سياسة إعادة محاولة مخصصة
    Ok::<_, anyhow::Error>(42)
})?;
```

### خيارات RetryPolicy

```rust
pub struct RetryPolicy {
    /// الحد الأقصى لمحاولات إعادة المحاولة
    pub max_attempts: usize,

    /// مدة التراجع الأولية (يتم تطبيق تراجع أسي)
    pub backoff: Duration,

    /// الحد الأقصى لمدة التراجع
    pub max_backoff: Duration,

    /// استخدام الاهتزاز في حساب التراجع أم لا
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            jitter: true,
        }
    }
}
```

## استخدام إعادة المحاولة في خطوط الأنابيب

### إعادة المحاولة داخل مرحلة

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // إعادة محاولة عملية الجلب
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // محاكاة جلب قد يفشل
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("خطأ في الشبكة"))
                } else {
                    Ok(format!("تم الجلب: {}", req))
                }
            }
        )
    }
]?;
```

### إعادة المحاولة على مستوى خط الأنابيب

لمزيد من التحكم، عالج إعادة المحاولة على مستوى المتصل:

```rust
fn process_with_retry(pool: &impl ThreadPool<Request = String, Response = String>, input: String) -> anyhow::Result<String> {
    retry_with(
        RetryPolicy {
            max_attempts: 5,
            backoff: Duration::from_millis(50),
            ..Default::default()
        },
        || {
            pool.send(input.clone())?;
            match pool.recv()? {
                Some(result) => Ok(result),
                None => Err(anyhow::anyhow!("انتهى خط الأنابيب")),
            }
        }
    )
}
```

## استراتيجيات استعادة الخطأ

### القيم الافتراضية

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  # الافتراضي إلى 0 عند الخطأ
    }
]?;
```

### تجميع الأخطاء

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<anyhow::Result<i32>> {
        req.into_iter()
            .map(|s| s.parse::<i32>().map_err(Into::into))
            .collect()
    },
    |req: Vec<anyhow::Result<i32>>| -> (i32, usize) {
        let (sum, errors) = req.into_iter().fold(
            (0, 0),
            |(sum, errs), r| match r {
                Ok(n) => (sum + n, errs),
                Err(_) => (sum, errs + 1),
            },
        );
        (sum, errors)
    }
]?;
```

### نمط قاطع الدائرة

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("قاطع الدائرة مفتوح"));
        }
        // معالجة الطلب
        Ok(format!("تمت المعالجة: {}", req))
    }
]?;
```

## مثال كامل

إليك مثال كامل يوضح معالجة الأخطاء وإعادة المحاولة:

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("إدخال غير صالح: {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // محاكاة عطل مؤقت
            if n % 3 == 0 {
                Err(anyhow::anyhow!("خطأ مؤقت"))
            } else {
                Ok(n * 2)
            }
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("نجح: {}", n),
                Err(e) => format!("فشل: {}", e),
            }
        }
    ]?;

    // إرسال عدة مدخلات
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // جمع النتائج
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## أفضل الممارسات

1. **استخدم `anyhow::Result`** لمعالجة أخطاء مرنة
2. **عين حدود إعادة المحاولة المناسبة** لتجنب حلقات لا نهائية
3. **استخدم التراجع الأسي** لعمليات الشبكة
4. **سجل الأخطاء بشكل مناسب** للتصحيح
5. **فكر في قواطع الدائرة** لاستدعاءات الخدمات الخارجية
6. **اجعل الأخطاء معلوماتية** — قم بتضمين سياق حول ما فشل
