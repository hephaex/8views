use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sc_archive::{encoding::is_image_entry, sort::natural_cmp};
use std::io::Write;
use tempfile::NamedTempFile;
use zip::ZipWriter;

// Minimal valid 1x1 PNG: 67 bytes
const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC, 0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// 지정된 페이지 수의 CBZ(ZIP) 파일을 임시 디렉토리에 생성한다.
/// 각 페이지는 MINIMAL_PNG로 "pageXXX.jpg" 이름으로 저장된다.
fn make_temp_cbz(page_count: usize) -> NamedTempFile {
    let mut temp = tempfile::Builder::new()
        .suffix(".cbz")
        .tempfile()
        .expect("Failed to create temp file");
    let mut writer = ZipWriter::new(&mut temp);

    for page_idx in 0..page_count {
        let filename = format!("page{:03}.jpg", page_idx);
        let options: zip::write::FileOptions<()> = zip::write::FileOptions::default();

        writer
            .start_file(&filename, options)
            .expect("Failed to start zip file entry");
        writer
            .write_all(MINIMAL_PNG)
            .expect("Failed to write PNG data");
    }

    writer.finish().expect("Failed to finish writing zip");
    temp
}

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

/// 50 페이지 CBZ 파일을 열고 엔트리 목록을 조회하는 성능을 측정한다.
/// 각 반복에서 임시 CBZ를 생성하고 open_archive() 호출 후 entries().len()을 수행한다.
fn bench_cbz_open_and_list_50pages(c: &mut Criterion) {
    c.bench_function("cbz_open_and_list_50pages", |b| {
        b.iter_with_setup(
            || make_temp_cbz(50),
            |cbz| {
                let path = cbz.path();
                let reader = sc_archive::open_archive(black_box(path)).unwrap();
                black_box(reader.entries().len())
            },
        );
    });
}

/// 50 페이지 CBZ 파일에서 첫 번째 이미지를 읽는 성능을 측정한다.
/// read_first_image()는 PartialReader를 사용하여 전체 아카이브를 열지 않으므로 빠를 것으로 예상된다.
fn bench_cbz_read_first_image_50pages(c: &mut Criterion) {
    c.bench_function("cbz_read_first_image_50pages", |b| {
        b.iter_with_setup(
            || make_temp_cbz(50),
            |cbz| {
                let path = cbz.path();
                let bytes = sc_archive::read_first_image(black_box(path)).unwrap();
                black_box(bytes.len())
            },
        );
    });
}

criterion_group!(
    benches,
    bench_natural_sort,
    bench_is_image_entry,
    bench_cbz_open_and_list_50pages,
    bench_cbz_read_first_image_50pages
);
criterion_main!(benches);
