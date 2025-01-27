# سمة ThreadPool

تحدد السمة `ThreadPool` الواجهة لجميع مجموعات خطوط الأنابيب التي تم إنشاؤها بواسطة الماكرو `pipe!`.

## تعريف السمة

```rust
pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&self, req: Self::Request) -> Result<()>;
    fn recv(&self) -> Result<Option<Self::Response>>;

    fn thread_usage(&self) -> Result<usize>;
    fn task_count(&self, id: impl ToString) -> Result<usize>;
}
```

## الطرق

### send

يرسل طلبًا إلى خط الأنابيب للمعالجة.

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**المعاملات:**
- `req` — الطلب المطلوب إرساله، يجب أن يتطابق مع نوع إدخال خط الأنابيب

**تُرجع:**
- `Result<()>` — Ok إذا تمت الإضافة إلى القائمة بنجاح، Err إذا فشل الإرسال

**مثال:**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

يتلقى نتيجة معالجة التالية من خط الأنابيب.

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**تُرجع:**
- `Ok(Some(response))` — نتيجة معالجة
- `Ok(None)` — انتهى خط الأنابيب
- `Err(...)` — حدث خطأ أثناء الاستلام

**مثال:**

```rust
loop {
    match pool.recv()? {
        Some(result) => println!("تم الاستلام: {}", result),
        None => break,
    }
}
```

### thread_usage

يُرجع عدد مؤشرات الترابط المستخدمة حاليًا بواسطة خط الأنابيب.

```rust
fn thread_usage(&self) -> Result<usize>
```

**تُرجع:**
- العدد الإجمالي لمؤشرات الترابط النشطة في جميع المراحل

**مثال:**

```rust
println!("مؤشرات الترابط النشطة: {}", pool.thread_usage()?);
```

### task_count

يُرجع عدد المهام المعلقة لمرحلة مسماة.

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**المعاملات:**
- `id` — اسم المرحلة (حسبما تم تعيينه بواسطة السمة `#[name(...)]`)

**تُرجع:**
- عدد المهام المنتظرة في قائمة هذه المرحلة

**مثال:**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("عمق قائمة parser: {}", pool.task_count("parser")?);
```

## معلمات النوع

### Request

نوع إدخال خط الأنابيب. هذا هو النوع الذي تقبله المرحلة الأولى.

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

نوع إخراج خط الأنابيب. هذا هو النوع الذي تُرجعه المرحلة الأخيرة.

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## دورة الحياة

يتبع خط الأنابيب دورة الحياة هذه:

1. **تم الإنشاء** — الماكرو `pipe!` يُرجع مجموعة جديدة
2. **نشط** — يمكنك `send()` الطلبات و `recv()` النتائج
3. **التصريف** — عند الإسقاط، تنهي المجموعة معالجة المهام المعلقة
4. **منتهي** — `recv()` تُرجع `None` عندما تنتهي المجموعة

### إيقاف التشغيل بأناقة

عند إسقاط المجموعة، فإنها:

1. تتوقف عن قبول الطلبات الجديدة
2. تنهي معالجة جميع المهام الموجودة في القائمة
3. تُوقف تشغيل جميع مجموعات مؤشرات الترابط بأناقة

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // تخرج المجموعة من النطاق وتتوقف عن التشغيل بأناقة
}
```

## المراقبة

استخدم طرق المراقبة لتتبع صحة خط الأنابيب:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("stage1")]
        |req: String| -> usize { req.len() },
        #[name("stage2")]
        |req: usize| -> String { req.to_string() }
    ]?;

    // إرسال العمل
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // مراقبة التقدم
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "مؤشرات الترابط: {}، Stage1 معلق: {}، Stage2 معلق: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
```
