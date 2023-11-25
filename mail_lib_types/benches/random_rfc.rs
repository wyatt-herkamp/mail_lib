use chumsky::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mail_lib_types::parsers::rfcs::rfc2822;

fn dot_atom_text(text: &str) {
    let _ = rfc2822::dot_atom_text().parse(text).unwrap();
}
fn bench_rfc2822(c: &mut Criterion) {
    c.bench_function("bench dot atom text", |b| {
        b.iter(|| dot_atom_text(black_box("this.random.library")))
    });
}

criterion_group!(random_rfcs, bench_rfc2822);
criterion_main!(random_rfcs);
