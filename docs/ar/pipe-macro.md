# الماكرو pipe!

الماكرو `pipe!` هو جوهر Ichika. يحول سلسلة من عمليات الإغلاق إلى خط أنابيب معالجة متعدد المراحل يعمل بالكامل.

## الصيغة الأساسية

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... المزيد من عمليات الإغلاق
]?;
```

تمثل كل عملية إغلاق مرحلة معالجة واحدة في خط الأنابيب الخاص بك.

### توقيعات عمليات الإغلاق

يجب أن تتبع كل عملية إغلاق هذه القواعد:

1. **قبول معامل واحد بالضبط** — المدخلات من المرحلة السابقة
2. **إرجاع نوع** — الذي يصبح المدخلات للمرحلة التالية
3. أن تكون `Clone + Send + 'static` — مطلوب لتنفيذ مجموعة مؤشرات الترابط

### أمثلة على التوقيعات

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // معالجة Result
}
```

## استنتاج الأنواع

يربط Ichika تلقائيًا نوع إخراج مرحلة واحدة بنوع إدخال المرحلة التالية:

```rust
let pool = pipe![
    |req: String| -> usize {        // المرحلة 1: String -> usize
        req.len()
    },
    |req: usize| -> String {         // المرحلة 2: usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // المرحلة 3: String -> bool
        !req.is_empty()
    }
]?;
```

## سمات المرحلة

يمكنك تكوين مراحل فردية باستخدام السمات:

### تكوين مجموعة مؤشرات الترابط

```rust
let pool = pipe![
    #[threads(4)]                    // استخدام 4 مؤشرات ترابط لهذه المرحلة
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // استخدام 2 مؤشر ترابط لهذه المرحلة
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### تكوين القائمة

```rust
let pool = pipe![
    #[queue(100)]                    // سعة قائمة 100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### المراحل المسماة

```rust
let pool = pipe![
    #[name("parser")]                # تسمية المرحلة للمراقبة
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

# الاستعلام عن عدد المهام للمرحلة المسماة
let count = pool.task_count("parser")?;
```

## خطوط الأنابيب المتفرعة

يمكنك إنشاء تفرع مشروط في خط الأنابيب الخاص بك:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // معالجة كل فرع
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("رقم: {}", n),
            Either::Right(s) => format!("سلسلة: {}", s),
        }
    }
]?;
```

## المراحل غير المتزامنة

مع الميزات المناسبة، يمكنك استخدام مراحل غير متزامنة:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // يتم تنفيذها في بيئة async
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## القيود العامة

يمكنك تعيين قيود عامة لخط الأنابيب بأكمله:

```rust
let pool = pipe![
    #[global_threads(8)]             # عدد مؤشرات الترابط الافتراضي لجميع المراحل
    #[global_queue(1000)]            # سعة القائمة الافتراضية
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## مثال كامل

إليك مثال أكثر واقعية يظهر ميزات متعددة:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        #[name("parse")]
        #[threads(2)]
        |req: String| -> anyhow::Result<i32> {
            log::info!("تحليل: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        #[threads(4)]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("معالجة: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("تنسيق: {}", n);
                    format!("النتيجة: {}", n)
                }
                Err(e) => {
                    log::error!("خطأ: {}", e);
                    format!("خطأ: {}", e)
                }
            }
        }
    ]?;

    // مراقبة استخدام مؤشرات الترابط
    println!("استخدام مؤشرات الترابط: {}", pool.thread_usage()?);

    Ok(())
}
```
