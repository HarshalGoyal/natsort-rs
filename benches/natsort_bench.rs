use criterion::{criterion_group, criterion_main, Criterion};
use natsort::{natsorted, natsorted_with, os_sorted, realsorted, NsFlags};

fn gen_strings(n: usize) -> Vec<String> {
    use fastrand::Rng;
    let mut rng = Rng::new();
    (0..n)
        .map(|_| {
            let prefix = match rng.u16(0..3) {
                0 => "file",
                1 => "img",
                _ => "item",
            };
            format!("{}{}.txt", prefix, rng.u16(1..999))
        })
        .collect()
}

fn bench_natsorted(c: &mut Criterion) {
    let sizes = [1_000, 10_000];
    for &size in &sizes {
        let data = gen_strings(size);
        c.bench_function(&format!("natsorted/{}", size), |b| {
            b.iter(|| natsorted(&data.iter().map(|s| s.as_str()).collect::<Vec<_>>()))
        });
    }
}

fn bench_realsorted(c: &mut Criterion) {
    let data: Vec<String> = (0..5_000)
        .map(|i| {
            let sign = if i % 3 == 0 { "-" } else { "" };
            format!("{}{}.{}", sign, i / 3, i % 10)
        })
        .collect();
    c.bench_function("realsorted/5k", |b| {
        b.iter(|| realsorted(&data.iter().map(|s| s.as_str()).collect::<Vec<_>>()))
    });
}

fn bench_os_sorted(c: &mut Criterion) {
    let data: Vec<String> = (0..5_000)
        .map(|i| format!("/dir/file{}.txt", i + 1))
        .collect();
    c.bench_function("os_sorted/5k", |b| {
        b.iter(|| os_sorted(&data.iter().map(|s| s.as_str()).collect::<Vec<_>>()))
    });
}

criterion_group!(benches, bench_natsorted, bench_realsorted, bench_os_sorted);
criterion_main!(benches);
