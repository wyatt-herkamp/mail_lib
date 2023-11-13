use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mail_lib_types::EmailAddress;

fn email(email: &str) {
    let _ = EmailAddress::new(email).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Test Normal Address email@example.com", |b| {
        b.iter(|| email(black_box("email@example.com")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
