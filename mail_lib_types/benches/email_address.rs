use chumsky::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mail_lib_types::{parsers::rfcs::rfc2822, EmailAddress};

fn email(email: &str) {
    let v = EmailAddress::new(email).unwrap();
    let _ = v.get_local().to_owned();
    let _ = v.get_domain().to_owned();
}
fn email_from_chumsky(email: &str) {
    let (local, domain) = rfc2822::addr_spec().parse(email).unwrap();
    let _ = unsafe { EmailAddress::new_unchecked_from_parts(local, domain) };
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Test Normal Address email@example.com", |b| {
        b.iter(|| email(black_box("email@example.com")))
    });
    c.bench_function("Test Normal Address email@example.com over chumsky", |b| {
        b.iter(|| email_from_chumsky(black_box("email@example.com")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
