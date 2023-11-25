use chumsky::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mail_lib_types::parsers::rfcs::rfc2822;

#[path = "../tests/data/data_types.rs"]
pub mod data_types;

fn mail_box(email: &str) {
    let _ = rfc2822::mailbox().parse(email).unwrap();
}
fn lettre_mailbox_bench(c: &mut Criterion) {
    c.bench_function("Bench Lettre Bench", |b| {
        b.iter(|| mail_box(black_box("\"Benchmark test\" <test@mail.local>")))
    });
}
fn bench_parse_mailbox(c: &mut Criterion) {
    let tests = data_types::build_valid_mailboxes();
    for test in tests {
        c.bench_function(format!("Parse MailBox {}", test.mailbox).as_str(), |b| {
            b.iter(|| mail_box(black_box(test.mailbox.as_str())))
        });
    }
}
criterion_group!(bench_mailbox, lettre_mailbox_bench, bench_parse_mailbox);
criterion_main!(bench_mailbox);
