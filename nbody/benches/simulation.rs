use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use nbody::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut sim = Simulation::new();
    sim.add_body(Body::new_lossy(0., 0., 10000.));
    sim.add_body({
        let mut body = Body::new_lossy(0., -200., 10.);
        body.velocity.x = Float::from_num(3);
        body
    });

    c.bench_function("sim 2", move |b| {
        b.iter_batched(|| sim.clone(), |mut sim| sim.step(), BatchSize::SmallInput)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
