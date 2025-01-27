# الميزات المتقدمة

يغطي هذا القسم الميزات والأساليب المتقدمة للاستفادة القصوى من Ichika.

## التكامل غير المتزامن

يدعم Ichika بيئات `tokio` و `async-std`. فعّل باستخدام الميزات:

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# أو
ichika = { version = "0.1", features = ["async-std"] }
```

### المراحل غير المتزامنة

امزج بين المراحل المتزامنة وغير المتزامنة بدون مشاكل:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // مرحلة متزامنة
        },
        async |req: usize| -> String {
            // مرحلة غير متزامنة - تُنفذ في بيئة tokio
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## منشئو مؤشرات الترابط المخصصون

يمكنك تخصيص كيفية إنشاء مؤشرات الترابط لكل مرحلة:

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // مكدس 2 ميجابايت
            .spawn(|| {
                // منطق مؤشر ترابط مخصص
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## المراقبة وإمكانية الرصد

### تتبع استخدام مؤشرات الترابط

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// الحصول على إجمالي عدد مؤشرات الترابط
let total_threads = pool.thread_usage()?;

// الحصول على عدد المهام المعلقة للمرحلة المسماة
let pending = pool.task_count("worker")?;

println!("مؤشرات الترابط: {}، معلق: {}", total_threads, pending);
```

### فحوصات السلامة

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## إدارة الموارد

### إيقاف التشغيل بأناقة

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// إنشاء مؤشر ترابط مراقبة
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // مراقبة صحة المجموعة
        thread::sleep(Duration::from_secs(1));
    }
});

// عند الانتهاء، عين running إلى false
running.store(false, Ordering::Relaxed);
// ستتوقف المجموعة عن التشغيل بأناقة عند الإسقاط
```

### اعتبارات الذاكرة

كل مرحلة لها قائمة محدودة. اضبط أحجام القائمة وفقًا لقيود الذاكرة الخاصة بك:

```rust
let pool = pipe![
    #[queue(100)]   # قائمة صغيرة للبيئات بقيود الذاكرة
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  # قائمة أكبر للمراحل عالية الأداء
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## أنماط خطوط الأنابيب

### Fan-out / Fan-in

عالج العناصر بالتوازي واجمع النتائج:

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<String> {
        req.into_iter()
            .filter(|s| !s.is_empty())
            .collect()
    },
    |req: Vec<String>| -> usize {
        req.len()
    }
]?;
```

### المعالجة ذات الحالة

استخدم `Arc<Mutex<T>>` للمراحل ذات الحالة:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("العناصر المعالجة: {}", *count);
        req.len()
    }
]?;
```

### التوجيه الشرطي

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("تسجيل دخول: {}", user),
            Event::Logout(user) => format!("تسجيل خروج: {}", user),
            Event::Message(from, msg) => format!("{}: {}", from, msg),
        }
    }
]?;
```

## ضبط الأداء

### ضبط حجم مجموعة مؤشرات الترابط

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  # يطابق عدد CPUs
    |req: String| -> usize {
        req.len()
    }
]?;
```

### المعالجة الدفعية

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  # استخدام rayon للمعالجة المتوازية
            .map(|s| s.len())
            .collect()
    }
]?;
```

## اختبار خطوط الأنابيب

### اختبار المراحل الوحدوية

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline() {
        let pool = pipe![
            |req: String| -> usize { req.len() },
            |req: usize| -> String { req.to_string() }
        ].unwrap();

        pool.send("test".to_string()).unwrap();
        let result = pool.recv().unwrap().unwrap();
        assert_eq!(result, "4");
    }
}
```

### اختبار التكامل

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
    }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // يجب أن تتعامل خطوط الأنابيب مع الأخطاء بأناقة
}
```

## أفضل الممارسات

1. **سمّ مراحلك** لمراقبة وتصحيح أفضل
2. **استخدم أعداد مؤشرات ترابط مناسبة** — لا تتجاوز CPU
3. **عين أحجام قائمة معقولة** لتقييد استخدام الذاكرة
4. **عالج الأخطاء بشكل صريح** — لا تتجاهل الأخطاء بصمت
5. **راقب استخدام الموارد** في الإنتاج
6. **اختبر مسارات الأخطاء** — ليس فقط المسارات السعيدة
7. **فكر في الضغط العكسي** — ماذا يحدث عندما يكون downstream بطيئًا؟
8. **استخدم async لمراحل I/O-bound**، sync لـ CPU-bound
