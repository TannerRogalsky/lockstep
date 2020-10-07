use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use nbody::*;

fn proto_disk<R: rand::Rng>(
    sim: &mut Simulation,
    rng: &mut R,
    count: usize,
    (origin_x, origin_y): (f32, f32),
    radius: f32,
) {
    for _ in 0..count {
        let rand = rng.gen::<f32>() * 2. * std::f32::consts::PI;
        let rand2 = rng.gen::<f32>();
        let x = (radius * rand2) * rand.cos();
        let y = (radius * rand2) * rand.sin();
        let mag = (x * x + y * y).sqrt();

        let mut body = Body::new_lossy(origin_x + x, origin_y + y, 1000.);
        body.velocity.x = Float::from_num(y * (mag / 7000.));
        body.velocity.y = Float::from_num(-x * (mag / 7000.));
        sim.add_body(body);
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut sim = Simulation::new();
    let mut rng = rand_pcg::Pcg64Mcg::new(0);
    proto_disk(&mut sim, &mut rng, 1000, (0., 0.), 400.);

    c.bench_function("sim proto disk", move |b| {
        b.iter_batched(|| sim.clone(), |mut sim| sim.step(), BatchSize::SmallInput)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
