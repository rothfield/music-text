use criterion::{black_box, criterion_group, criterion_main, Criterion};
use music_text::parse_document;

fn benchmark_simple_notation(c: &mut Criterion) {
    c.bench_function("parse simple |123", |b| {
        b.iter(|| parse_document(black_box("|123")))
    });
}

fn benchmark_complex_notation(c: &mut Criterion) {
    let complex_input = "|1-2-3 4-5-6|7-8-9 |SRG |CDE |";
    c.bench_function("parse complex notation", |b| {
        b.iter(|| parse_document(black_box(complex_input)))
    });
}

fn benchmark_very_long_input(c: &mut Criterion) {
    let long_input = "|1234567890 1234567890 1234567890 1234567890 1234567890|";
    c.bench_function("parse long input", |b| {
        b.iter(|| parse_document(black_box(long_input)))
    });
}

fn benchmark_multi_stave(c: &mut Criterion) {
    let multi_stave = "===\n|123\n|456\n|789\n===";
    c.bench_function("parse multi-stave", |b| {
        b.iter(|| parse_document(black_box(multi_stave)))
    });
}

fn benchmark_with_annotations(c: &mut Criterion) {
    let with_annotations = "____\n|123\n....";
    c.bench_function("parse with annotations", |b| {
        b.iter(|| parse_document(black_box(with_annotations)))
    });
}

criterion_group!(
    benches, 
    benchmark_simple_notation,
    benchmark_complex_notation,
    benchmark_very_long_input,
    benchmark_multi_stave,
    benchmark_with_annotations
);
criterion_main!(benches);