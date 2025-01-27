# البدء

سيساعدك هذا الدليل على البدء مع Ichika، من التثبيت إلى خط الأنابيب الأول الخاص بك.

## التثبيت

أضف Ichika إلى ملف `Cargo.toml` الخاص بك:

```toml
[dependencies]
ichika = "0.1"
```

### أعلام الميزات

يدعم Ichika بيئات تشغيل غير متزامنة مختلفة عبر أعلام الميزات:

```toml
# لدعم tokio (الافتراضي)
ichika = { version = "0.1", features = ["tokio"] }

# لدعم async-std
ichika = { version = "0.1", features = ["async-std"] }

# لكلتا البيئتين
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## خط الأنابيب الأول

لنقم بإنشاء خط أنابيب بسيط لمعالجة السلاسل النصية:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // تعريف خط أنابيب مكون من 3 مراحل
    let pool = pipe![
        // المرحلة 1: تحليل السلسلة إلى رقم
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("فشل التحليل: {}", e))
        },
        // المرحلة 2: مضاعفة الرقم
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // المرحلة 3: التحويل مرة أخرى إلى سلسلة
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("خطأ: {}", e))
        }
    ]?;

    // معالجة بعض البيانات
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // جمع النتائج
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("النتيجة: {}", result);
        }
    }

    Ok(())
}
```

## فهم الأساسيات

### الماكرو pipe!

ينشئ الماكرو `pipe!` سلسلة من مراحل المعالجة. كل مرحلة:

1. تستقبل المدخلات من المرحلة السابقة (أو من استدعاء `send()` الأولي)
2. تعالج البيانات في مجموعة مؤشرات الترابط
3. تمرر النتيجة إلى المرحلة التالية

### انتشار الأنواع

يستنتج Ichika تلقائيًا الأنواع التي تتدفق عبر خط الأنابيب الخاص بك:

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### معالجة الأخطاء

يمكن لكل مرحلة إرجاع `Result`، وتنتشر الأخطاء تلقائيًا:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // أو تعامل مع الخطأ بشكل مناسب
    }
]?;
```

## الخطوات التالية

- تعرف على المزيد حول [ماكرو pipe!](./pipe-macro.md)
- افهم [سمة ThreadPool](./threadpool-trait.md)
- استكشف [معالجة الأخطاء](./error-handling.md) بعمق
- شاهد المزيد من [الأمثلة](./examples.md)
