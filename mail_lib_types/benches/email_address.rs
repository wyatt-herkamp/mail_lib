use chumsky::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mail_lib_types::{parsers::rfcs::rfc2822, EmailAddress};
#[path = "../tests/data/data_types.rs"]
pub mod data_types;

fn email(email: &str) {
    let _ = EmailAddress::new(email).unwrap();
}
fn email_from_chumsky(email: &str) {
    let _ = rfc2822::addr_spec().parse(email).unwrap();
}
/// Chumsky will be slower. The regular EmailAddress is just validating the email address while Chumsky is parsing the entire email.
fn bench_validate_emails(c: &mut Criterion) {
    for test in data_types::build_valid_tests() {
        c.bench_function(format!("Validate Email: `{}`", test.email).as_str(), |b| {
            b.iter(|| email(black_box(&test.email)))
        });
    }
}

fn bench_benchmark_chumsky(c: &mut Criterion) {
    for test in data_types::build_valid_tests() {
        c.bench_function(format!("Parse Email: `{}`", test.email).as_str(), |b| {
            b.iter(|| email_from_chumsky(black_box(&test.email)))
        });
    }
}

criterion_group!(bench_email_address, bench_validate_emails, bench_benchmark_chumsky);
criterion_main!(bench_email_address);
