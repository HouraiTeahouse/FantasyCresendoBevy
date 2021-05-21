use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fc::geo::Capsule3D;

fn criterion_benchmark(c: &mut Criterion) {
    let sphere_a = Capsule3D::sphere((0.0, 0.0, 0.0), 3.0);
    let sphere_b = Capsule3D::sphere((11.0, 0.0, 0.0), 8.0);
    let capsule_a = Capsule3D {
        start: (-3.0, 0.0, 0.0).into(),
        end: (3.0, 0.0, 0.0).into(),
        radius: 4.0,
    };
    let capsule_b = Capsule3D {
        start: (10.0, -3.0, 0.0).into(),
        end: (3.0, 0.0, 0.0).into(),
        radius: 3.0,
    };
    c.bench_function("capsule intersect sphere-sphere", |bench| {
        bench.iter(|| sphere_a.intersects(&sphere_b))
    });
    c.bench_function("capsule intersect capsule-capsule", |bench| {
        bench.iter(|| capsule_a.intersects(&capsule_b))
    });
    c.bench_function("capsule intersect capsule-sphere", |bench| {
        bench.iter(|| capsule_a.intersects(&sphere_b))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
