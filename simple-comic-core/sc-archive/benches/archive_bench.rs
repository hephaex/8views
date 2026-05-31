use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sc_archive::{encoding::is_image_entry, sort::natural_cmp};

// NOTE: 실제 아카이브 파일 열기 벤치마크는 Sprint 6에서 픽스처 파일 추가 예정.

/// 100개의 합성 파일명을 자연 정렬 알고리즘으로 정렬하는 속도를 측정한다.
fn bench_natural_sort(c: &mut Criterion) {
    // 숫자 패딩이 다양한 100개 파일명 생성 (정렬 알고리즘이 non-trivial한 비교를 수행하도록)
    let filenames: Vec<String> = (0..100)
        .map(|i| {
            // 뒤섞인 순서로 생성해 실제 정렬 작업이 발생하도록 한다
            let n = (i * 37 + 13) % 100;
            format!("chapter{n}/page{i:03}.jpg")
        })
        .collect();

    let mut group = c.benchmark_group("archive_sort");

    group.bench_with_input(
        BenchmarkId::new("natural_sort", "100_filenames"),
        &filenames,
        |b, names| {
            b.iter(|| {
                let mut sorted = names.clone();
                // black_box 로 컴파일러가 정렬을 제거하는 최적화를 방지한다
                sorted.sort_by(|a, b| natural_cmp(black_box(a), black_box(b)));
                black_box(sorted)
            });
        },
    );

    group.finish();
}

/// 1000번 파일명 확장자 필터링이 이미지 엔트리 판별에 걸리는 시간을 측정한다.
fn bench_is_image_entry(c: &mut Criterion) {
    // 이미지와 비이미지 파일명을 섞어 분기 예측 효과를 배제한다
    let entries: Vec<&str> = vec![
        "cover.jpg",
        "page001.png",
        "readme.txt",
        "page002.jpeg",
        "metadata.xml",
        "spread.webp",
        "thumbs.db",
        "page003.gif",
        "chapter.md",
        "image.bmp",
    ];

    let mut group = c.benchmark_group("archive_filter");

    group.bench_function("is_image_entry_1000x", |b| {
        b.iter(|| {
            let mut count = 0usize;
            for _ in 0..100 {
                for entry in &entries {
                    // black_box 로 인라인 상수 최적화를 방지한다
                    if is_image_entry(black_box(entry)) {
                        count += 1;
                    }
                }
            }
            black_box(count)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_natural_sort, bench_is_image_entry);
criterion_main!(benches);
