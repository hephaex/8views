# 8views — Rust 리팩토링 진행 상황

> 시작: 2026-06-01
> 현재 Phase: **Phase 8 완료** (Sprint 22 — QuickLook Swift 신형 API Rust 교체)

---

## 전체 진행률

```
Phase 1: 설정          [x] 2/2 sprint (완료)
Phase 2: 아카이브 엔진  [x] 4/4 sprint (완료)
Phase 3: 이미지 파이프라인 [x] 4/4 sprint (완료)
Phase 4: 세션 스토리지     [~] 1/3 sprint (Sprint 10 — pages API)
Phase 5: Swift FFI     [x] 3/3 sprint (Sprint 6+7+8 — 완료)
Phase 6: UI 배선        [x] 6/6 완료 (Sprint 11-16)
Phase 7: OCR 통합       [ ] 0/2 sprint
Phase 8: QuickLook     [x] 2/2 sprint (완료 — Sprint 20-22)
Phase 9: 최종 검증      [~] 진행중 (Sprint 17-22)
────────────────────────────────────
합계: Sprint 22 완료 (2026-06-02)
```

---

## Phase 1: 프로젝트 설정

### Sprint 1: Cargo Workspace 구성 ✅ (2026-06-01)
- [x] `8views-core/` Cargo workspace 생성
- [x] 크레이트 구조 설계: `sc-core`, `sc-archive`, `sc-image`, `sc-storage`, `sc-ffi`
- [x] uniffi 0.29 + zip/sevenz/tar/image/rusqlite 의존성 설정
- [x] CI 설정: GitHub Actions `cargo test`, `cargo clippy`, `cargo fmt`
- [x] `cargo check/test/clippy/fmt` 전체 통과 (12 tests)
- [x] 커밋: 8833a8d

### Sprint 2: 개발 환경 & 벤치마크 기준선 ✅ (2026-06-01)
- [x] 크로스 컴파일 검증: arm64 + x86_64 → `libeightviews-universal.a` 11MB
- [x] criterion 벤치마크 하네스: archive (sort/filter), image (scale/compositor/cache)
- [x] 테스트 픽스처: `tests/common/mod.rs` — make_cbz / make_tar_gz 생성기
- [x] 통합 테스트 4개 (CBZ 목록/읽기/정렬, TAR.GZ 목록)
- [x] 커밋: f034d47

---

## Phase 2: 아카이브 엔진

### Sprint 3: Phase 2 내실화 ✅ (2026-06-01)
- [x] `PartialReader` — `read_first_image()` (DTPartialArchiveParser 이식): ZIP/TAR/7z/folder 4가지 경로
- [x] `RarArchive` — unrar-ng 0.7 typestate cursor API, `.rar`/`.cbr` 라우팅
- [x] 7z 픽스처 (`make_sevenz`) + 통합 테스트 (목록, 읽기)
- [x] 폴더 픽스처 (`make_folder`) + 통합 테스트 (목록, 읽기)
- [x] PartialReader 통합 테스트 4개 (CBZ/TAR.GZ/folder/7z 첫 이미지 추출)
- [x] 커밋: 8ac7425

### Sprint 4: TAR 변형 + 엣지케이스 + 벤치마크 ✅ (2026-06-01)
- [x] TAR.BZ2 + TAR.XZ 픽스처(`make_tar_bz2`, `make_tar_xz`) + 통합 테스트 4개
- [x] 엣지케이스 5개: 빈 아카이브, 비이미지 필터링, 혼합 필터링, 없는 경로, 빈 폴더
- [x] 아카이브 벤치마크 기준선: `cbz_open_and_list_50pages`, `cbz_read_first_image_50pages`
- [x] 커밋: 5e13d92

### Sprint 5: Phase 2 내실화 + FFI 준비 ✅ (2026-06-01)
- [x] `detect.rs`: magic byte 기반 포맷 탐지 (ZIP/7z/RAR/TAR.gz/bz2/xz) + 8개 단위 테스트
  - `open_archive()` fallback — 확장자 없거나 잘못된 파일 자동 탐지
- [x] `eightviews.udl`: uniffi 인터페이스 스켈레톤 (archive 3 + session 3 + util)
- [x] `sc-ffi/src/lib.rs`: 타입 검증 스텁 7개 (Sprint 6 scaffolding 전환 준비)
- [x] PartialReader TAR.BZ2 + TAR.XZ 통합 테스트 추가
- [x] 커밋: f96b4f1

### Sprint 6: uniffi 0.29 스캐폴딩 활성화 ✅ (2026-06-01)
- [x] `build.rs` → `uniffi::generate_scaffolding("src/eightviews.udl")` 활성화
- [x] `lib.rs` → `uniffi::include_scaffolding!("eightviews")` + UDL 함수 구현
- [x] `ArchiveEntryRecord` / `SessionStateRecord` 구조체 (UDL dictionary 대응)
- [x] `ScError` flat enum (UDL [Error] enum 대응)
- [x] 아카이브 API: `archive_list_pages`, `archive_read_page`, `archive_read_first_image`
- [x] 세션 API: `session_load`, `session_save`, `session_delete`
- [x] 커밋: 5066efe

---

## Phase 3: 이미지 파이프라인

### Sprint 7: Swift 바인딩 + 이미지 파이프라인 ✅ (2026-06-01)
- [x] `tools/uniffi-bindgen`: workspace binary crate (cargo run --bin uniffi-bindgen)
- [x] Swift 바인딩 생성: `eightviews.swift` 1,084줄, `eightviewsFFI.h`, `module.modulemap`
- [x] `EightViewsCore/` Swift Package: Package.swift + Universal lib (15MB)
  - XCTest 4개: version, archive_missing, session_default, session_CRUD
- [x] sc-image 통합 테스트 10개: load, scale 3종, compositor 2종, cache 2종, roundtrip
  - MINIMAL_PNG CRC 수정 (image 크레이트 strict PNG 검증)
- [x] `thumbnail.rs`: `ThumbnailGenerator` — rayon par_iter 병렬, sorted stable 출력
- [x] 커밋: 3f18285

### Sprint 8: Swift XCTest 통과 + Rotation + 벤치마크 ✅ (2026-06-01)
- [x] Package.swift 링커 수정 — libbz2 + liblzma(Homebrew) + libc++ 추가
- [x] Swift XCTest 4/4 통과 (version, archive_err, session_default, session_CRUD)
- [x] `SessionState::default()` zoom_level 0.0 → 1.0 수정
- [x] `rotate.rs`: `apply_rotation()` + `scale_then_rotate()` — R90/R180/R270
  - 8개 테스트 (5 unit + 3 integration)
- [x] sc-image 벤치마크: thumbnail parallel 10/50 + serial 50 비교
- [x] EightViewsCore .gitignore 추가 (.build/ 제외)
- [x] 커밋: 295be1e, d37108f

### Sprint 9: 파이프라인 E2E + 썸네일 FFI + App Support DB ✅ (2026-06-01)
- [x] `sc-ffi/tests/pipeline_test.rs`: 5개 E2E 통합 테스트
  archive→image→scale→rotate→thumbnail 전체 체인
- [x] `eightviews.udl`: `generate_thumbnail`, `generate_archive_thumbnails` + `ThumbnailRecord`
- [x] `sc-ffi/src/lib.rs`: 썸네일 API 구현 (rayon 병렬, RGBA 바이트 반환)
- [x] 세션 DB 경로: `$TMPDIR` → `~/Library/Application Support/8views/sessions.sqlite`
- [x] `sc-ffi/Cargo.toml`: `dirs`, `image` 의존성 추가
- [x] `crate-type` += `rlib` (통합 테스트 링킹 지원)
- [x] 커밋: 6f6ec9f

### Sprint 10: Phase 3 완결 + Phase 4 시작 ✅ (2026-06-01)
- [x] Swift 바인딩 재생성 (1,253줄 — thumbnail API 포함)
- [x] libeightviews-universal.a 재빌드 (dirs + image deps)
- [x] JPEG/WebP/GIF/BMP 포맷 로딩 통합 테스트 4개 (round-trip 검증)
  → Phase 3 모든 포맷 커버리지 완성
- [x] sc-storage `page_metadata` 테이블 (SCHEMA_V2)
  - `PageRecord` struct + `upsert_page_metadata` / `get_page_metadata` / `clear_page_metadata`
  - 4개 단위 테스트 (CASCADE delete 포함)
- [x] 커밋: 4f7cc4c

---

## Phase 4: 세션 스토리지

### Sprint 11: SQLite 스키마 설계
- [ ] Core Data 6 엔티티 → SQLite 테이블 설계
  ```sql
  sessions, groups, pages (hierarchy via parent_id)
  ```
- [ ] `rusqlite_migration` 마이그레이션 v0→v1
- [ ] `SessionManager`: create, load, save, delete
- [ ] 단위 테스트: CRUD 전체

### Sprint 12: 세션 상태 + xattr
- [ ] 세션 상태 저장: zoom, rotation, two_page_spread, page_order, scroll_position
- [ ] `XattrStore`: 파일 확장 속성 읽기/쓰기 (`xattr` 크레이트)
- [ ] 단위 테스트: 상태 저장/복원 round-trip

### Sprint 13: Core Data 마이그레이션
- [ ] Core Data 바이너리 → SQLite 변환 도구
- [ ] 기존 세션 데이터 손실 없는 마이그레이션 검증
- [ ] uniffi UDL: 세션 API 노출
- [ ] 통합 테스트: 마이그레이션 후 모든 세션 접근 가능

---

## Phase 5: Swift FFI 통합

### Sprint 14: uniffi 바인딩 생성
- [ ] UDL 파일 전체 작성 (`eightviews.udl`)
- [ ] `cargo build` → `libeightviews.a` + Swift 바인딩 파일 자동 생성
- [ ] Swift Package에서 빌드 검증

### Sprint 15: Xcode 통합
- [ ] Xcode 프로젝트 링크 설정 (`-leightviews`, `-lc++`)
- [ ] Build Phase: Rust 빌드 스크립트 추가
- [ ] Bridging header 업데이트
- [ ] Swift 단위 테스트: Rust 함수 호출 검증

### Sprint 16: 데이터 타입 매핑
- [ ] Rust 타입 ↔ Swift 타입 완전 매핑
- [ ] 에러 전파: Rust `anyhow::Error` → Swift `Error`
- [ ] 메모리 소유권 검증 (ARC + Rust 라이프타임)
- [ ] 스레드 안전성 검증

---

## Phase 6: UI 배선

### Sprint 17: 앱 델리게이트 배선
- [ ] `EightViewsAppDelegate.m` — 아카이브 오픈 → Rust `open_archive()`
- [ ] 파일 연결 (UTI) 검증
- [ ] 드래그 앤 드롭 검증

### Sprint 18~19: 세션 윈도우 컨트롤러 배선
- [ ] `TSSTSessionWindowController.m` — 페이지 전환 → Rust 캐시
- [ ] 세션 상태 저장/복원 → Rust `SessionManager`
- [ ] 줌/회전/페이지 순서 → Rust 상태

### Sprint 20: 페이지 뷰 배선
- [ ] `TSSTPageView.m` — 이미지 렌더링 → Rust 파이프라인
- [ ] 두 페이지 합성 → Rust `Compositor`
- [ ] 픽셀 동일성 검증

### Sprint 21: 썸네일 뷰 배선
- [ ] `TSSTThumbnailView.swift` — 썸네일 → Rust 병렬 생성
- [ ] 프로그레스 바 → Rust 세션 상태

### Sprint 22: 전체 UI 검증
- [ ] 모든 메뉴 항목 동작 확인
- [ ] 환경설정 창 저장/불러오기
- [ ] 전체화면 모드 + Touch Bar

---

## Phase 7: OCR 통합

### Sprint 23: OCR 검색 엔진 이식
- [ ] `OCRFind.m` 검색 로직 → Rust (`aho-corasick`)
- [ ] 텍스트 인덱스 Rust 구조체로 관리
- [ ] Vision 결과(Swift) → Rust 인덱스 업데이트 경로

### Sprint 24: OCR UI 검증
- [ ] 텍스트 선택 정확도 기존 대비 비교
- [ ] Find 다이얼로그 (OCRFindViewController) 동작 확인
- [ ] 회전/줌 상태에서 텍스트 선택 확인

---

## Phase 8: QuickLook 플러그인

### Sprint 25: 썸네일 익스텐션
- [ ] `ThumbnailProvider.swift` → Rust `PartialReader` + `ImageLoader` 위임
- [ ] Finder 썸네일 검증 (CBZ/CBR/7z)

### Sprint 26: 미리보기 익스텐션
- [ ] `PreviewProvider.swift` → Rust 아카이브 첫 이미지 추출 위임
- [ ] 스페이스바 미리보기 검증

---

## Phase 9: 최종 검증 및 정리

### Sprint 27: 성능 + 메모리 검증
- [ ] 전체 벤치마크 실행 (PLAN.md 기준 충족 확인)
- [ ] Instruments Leaks: 메모리 누수 0
- [ ] Instruments Allocations: 피크 메모리 < 200MB
- [ ] 미사용 Objective-C 코드 제거

### Sprint 28: Core Data 제거 + 릴리즈
- [ ] Core Data 프레임워크 의존성 완전 제거
- [ ] `Sessions_DataModel.xcdatamodeld` 삭제
- [ ] CHANGELOG 업데이트
- [ ] v2.0.0 태그 생성

---

## 완료된 스프린트

### Sprint 1 — Cargo Workspace 구성 (2026-06-01)

| 항목 | 결과 |
|------|------|
| tests | 12 pass / 0 fail |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | 8833a8d |

**생성된 모듈:**
- `sc-core`: ScaleMode, Rotation, PageOrder, SortOrder, ImageMetadata, ScError
- `sc-archive`: ArchiveReader trait, ZipArchive, TarArchive, SevenZArchive, FolderReader, 자연 정렬, 인코딩 감지
- `sc-image`: ImageLoader, ScaleOptions, Compositor (두 페이지 합성), ImageCache (LRU)
- `sc-storage`: SessionManager (SQLite CRUD), xattr_store, migration schema
- `sc-ffi`: static lib 스캐폴딩, sc_version() FFI 함수

**주의:** uniffi 0.29 사용 (최신 0.31.1과 의존성 충돌로 락됨 — Sprint 14 FFI 통합 시 업그레이드 검토)

### Sprint 2 — 개발 환경 & 벤치마크 기준선 (2026-06-01)

| 항목 | 결과 |
|------|------|
| tests | 16 pass / 0 fail (12 unit + 4 integration) |
| clippy | 경고 0 |
| fmt | 통과 |
| universal lib | 11MB (arm64 + x86_64) |
| 커밋 | f034d47 |

**추가된 내용:**
- Universal static lib (`lipo -create`) 빌드 검증 완료
- criterion 벤치마크 5개: `natural_sort`, `is_image_entry`, `scale_fit_window`, `two_page_spread`, `cache_insert_evict`
- `tests/common/mod.rs` 픽스처: `make_cbz`, `make_cbz_with_names`, `make_tar_gz`
- 통합 테스트: `cbz_open_and_list`, `cbz_read_entry_returns_bytes`, `cbz_natural_sort_order`, `tar_gz_open_and_list`

**학습:** `#[cfg(test)]`로 게이트한 `src/` 모듈은 `tests/` 통합 테스트에서 접근 불가 → `tests/common/mod.rs` 패턴 사용

### Sprint 3 — Phase 2 내실화 (2026-06-01)

| 항목 | 결과 |
|------|------|
| tests | 34 pass / 0 fail (14 unit + 12 integration + 1 doctest + 7 other) |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | 8ac7425 |

**추가된 내용:**
- `partial_reader.rs`: `read_first_image()` — 모든 포맷 최적화 첫 이미지 추출
- `rar_archive.rs`: unrar-ng 0.7 RarArchive (typestate cursor pattern)
- `tests/common/mod.rs` 픽스처 확장: `make_folder`, `make_sevenz`
- 통합 테스트 8개 추가: folder(2), 7z(2), PartialReader(4)

**결정:** PLAN.md의 `compress-tools`로 RAR 계획을 `unrar-ng`로 변경 — libarchive가 brew 설치됐지만 pkg-config 미등록이라 cross-compile 불안정. unrar-ng는 C++ 소스 번들로 의존성 없음.

---

## 결정 기록

| 날짜 | 결정 | 근거 |
|------|------|------|
| 2026-06-01 | 하이브리드 아키텍처 채택 | macOS UI 바인딩 미성숙, 점진적 교체 가능 |
| 2026-06-01 | uniffi로 Swift 바인딩 | Mozilla 검증, Swift Package 통합 용이 |
| 2026-06-01 | unrar-ng로 RAR (compress-tools 계획 변경) | libarchive pkg-config 미등록, cross-compile 불안정 |
| 2026-06-01 | Vision 프레임워크 Swift 유지 | macOS 전용, Rust 바인딩 없음 |

### Sprint 4 — TAR 변형 + 엣지케이스 + 벤치마크 (2026-06-01)

| 항목 | 결과 |
|------|------|
| tests | 43 pass / 0 fail (+9 신규) |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | 5e13d92 |

**추가된 내용:**
- 픽스처: `make_tar_bz2`, `make_tar_xz` → TAR 변형 전체 커버
- 엣지케이스 테스트 5개: 빈/비이미지/혼합/없는경로/빈폴더
- 벤치마크: 50페이지 CBZ open+list, PartialReader 기준선

### Sprint 5 — Phase 2 내실화 + FFI 준비 (2026-06-01)

| 항목 | 결과 |
|------|------|
| tests | 61 pass / 0 fail (+18 신규) |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | f96b4f1 |

**추가된 내용:**
- `detect.rs`: 매직 바이트 탐지 — ZIP/7z/RAR/TarGz/TarBz2/TarXz 6종, 단위 테스트 8개
- `eightviews.udl`: uniffi 인터페이스 8개 함수, 2개 dictionary, 1개 error enum
- `sc-ffi/src/lib.rs`: 타입 경계 검증 스텁 + 단위 테스트 7개 (Sprint 6 scaffolding 준비)
- PartialReader TAR.BZ2/XZ 통합 테스트 2개

**Phase 2 포맷 커버리지 (최종):**
ZIP/CBZ ✓ | TAR.GZ/BZ2/XZ ✓ | 7z ✓ | folder ✓ | RAR/CBR ✓ | magic byte fallback ✓

### Sprint 6 — uniffi 0.29 스캐폴딩 활성화 (2026-06-01)

| 항목 | 결과 |
|------|------|
| tests | 61 pass / 0 fail (변화 없음 — 스캐폴딩은 컴파일 타임) |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | 5066efe |

**핵심 학습:**
- uniffi UDL 방식에서 `#[derive(uniffi::Record)]` + `include_scaffolding!()` 동시 사용 금지
  - generate_scaffolding이 이미 trait impl을 생성하므로 E0119 충돌 발생
  - UDL 방식에서는 Rust 타입에 uniffi derive 불필요
- `[Error] enum ScError { "Archive", ... }` → Rust flat enum (데이터 없음) 필수
  - 에러 정보는 Display impl 메시지로 전달
- `session_delete`는 UDL에서 `void` (throws 없음) → Rust `()` 반환, 에러 silently 무시

**Phase 2 완료. Phase 5(Swift FFI) Sprint 6에서 시작됨.**

### Sprint 7 — Swift 바인딩 + 이미지 파이프라인 (2026-06-01)

| 항목 | 결과 |
|------|------|
| tests | 77 pass / 0 fail (+16 신규) |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | 3f18285 |

**추가된 내용:**
- uniffi-bindgen workspace binary → Swift 바인딩 생성 파이프라인 확립
- EightViewsCore Swift Package (SPM) — Xcode 통합 준비 완료
- sc-image 통합 테스트 10개 (실제 PNG 바이트 기반)
- ThumbnailGenerator: rayon 병렬, Lanczos3, stable 정렬 출력

**부수 수정:** MINIMAL_PNG 상수의 잘못된 CRC → image 크레이트의 strict PNG 검증 통과

### Sprint 8 — Swift XCTest + Rotation + 벤치마크 (2026-06-01)

| 항목 | 결과 |
|------|------|
| Rust tests | 87 pass / 0 fail (+10 신규) |
| Swift XCTest | 4/4 pass |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | 295be1e + d37108f |

**핵심 성과:**
- Swift FFI Phase 5 완료 (3/3 sprint) — Xcode 통합 준비 100%
- 링커 이슈 해결: libbz2(SDK) + liblzma(Homebrew) + libc++(C++) 명시적 링킹
- rotation 파이프라인: RGBA 통일 변환 후 imageops rotate90/180/270 적용
- `zoom_level` 기본값 0.0→1.0 수정 (세션 미존재 시 올바른 100% 줌 반환)

### Sprint 9 — E2E 파이프라인 + 썸네일 FFI + App Support DB (2026-06-01)

| 항목 | 결과 |
|------|------|
| tests | 92 pass / 0 fail (+5 신규 E2E) |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | 6f6ec9f |

**핵심 학습:**
- `sc-ffi`에 `rlib` crate-type 추가 필수 — `staticlib`/`cdylib`만으로는 통합 테스트에서 `extern crate eightviews` 링크 불가
- UDL의 `[ByRef] bytes` → Rust `&[u8]` (소유권 이전 없이 바이트 슬라이스 전달)

### Sprint 10 — Phase 3 완결 + Phase 4 시작 (2026-06-01)

| 항목 | 결과 |
|------|------|
| Rust tests | 100 pass / 0 fail (+8 신규) |
| Swift XCTest | 4/4 pass |
| clippy | 경고 0 |
| fmt | 통과 |
| 커밋 | 4f7cc4c |

**마일스톤:**
- **Phase 3 (이미지 파이프라인) 4/4 완료** — 모든 포맷 커버리지, FFI 노출
- **Phase 4 시작** — page_metadata 테이블 + CRUD API
- Swift 바인딩 1,253줄 (thumbnail API 포함)

### Sprint 11 — Phase 6 시작: AppDelegate 아카이브 감지 배선 (2026-06-01)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass (변화 없음) |
| clippy | 경고 0 |
| 커밋 | fce18ee |

**배선 완료 (1/6):**
- `sc-archive::is_archive_supported()` 추가 — 확장자 우선, magic bytes 폴백
- `sc-ffi`: UDL `archive_is_supported()` + C FFI `sc_archive_is_supported()`
- `sc_extras.h`: 수동 C 헤더 (ObjC 직접 호출용)
- `EightViewsAppDelegate.m` 3곳: `[TSSTManagedArchive archiveExtensions]` → `sc_archive_is_supported()`

**개선점:** 확장자 없는 아카이브도 magic bytes로 정확히 감지 (기존 ObjC 방식 대비)

### Sprint 12 — Phase 6: requestDataForPageIndex → Rust archive_read_page (2026-06-01)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass (sc-ffi: 11 pass) |
| clippy | 경고 0 |
| 커밋 | bf88013 + ab108fc |

**배선 완료 (2/6 AppDelegate-level):**
- `sc_archive_read_page()` + `sc_free_bytes()` C FFI 추가 (no_mangle, heap ownership)
- `SCArchiveError` enum + `SCArchiveErrorDomain` 매크로 → `sc_extras.h`
- `TSSTManagedArchive.requestDataForPageIndex:` — XADMaster 교체 완료
  - solidDirectory 디스크 캐시 제거 (Rust libarchive 직접 처리)
  - `groupLock` 불필요 (per-call archive handle, 공유 상태 없음)
- libeightviews.a universal 재빌드 (arm64+x86_64)
- 4개 신규 Rust 단위 테스트 (success/out-of-range/null-path/free-null)

**리뷰에서 발견된 이슈:**
- **TODO**: solid RAR archives O(n) 재해제 문제 → Sprint 13에서 페이지 캐시 전략 수립
- **TODO**: 매 페이지마다 archive open/close → 아카이브 핸들 캐시 필요 (Sprint 13)

### Sprint 13 — Phase 6: nestedArchiveContents → Rust ScPageList (2026-06-01)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass (sc-ffi: 16 pass, +5 신규) |
| clippy | 경고 0 |
| 커밋 | 72e55c9 |

**배선 완료 (3/6 Phase 6):**
- `ScPageList` opaque 핸들 패턴: 아카이브 한 번 오픈 → 모든 페이지 메타데이터 캐시
- `sc_archive_open_pages` / `sc_page_list_count/name/size` / `sc_archive_pages_free` 5개 C FFI
- `nestedArchiveContents` Phase 1 (이미지 열거) → Rust 교체 완료
- **핵심 버그 수정**: Sprint 12의 인덱스 불일치 문제 해결
  - 기존: XADMaster raw counter(전체 엔트리) → Rust 인덱스(이미지만) 불일치
  - 수정: Rust ScPageList iteration index `i` → Core Data index 저장

**Phase 2 (nested archive/PDF) XADMaster 잔존 (edge case):**
- Sprint 14 TODO: `sc_archive_read_entry_bytes(path, index)` 일반 엔트리 API 추가

### Sprint 14 — Phase 6: 세션 저장/복원 → Rust SessionManager (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass (sc-ffi: 20 pass, +4 신규) |
| clippy | 경고 0 |
| 커밋 | 72d19de |

**배선 완료 (4/6 Phase 6):**
- `ScSessionState` repr(C) 구조체 + 3개 C FFI (load/save/delete)
- `sc_session_load`: 레코드 없으면 false 반환 (Default state와 구별 가능)
- `TSSTSessionWindowController`:
  - `restoreSession`: Rust 레코드 있으면 Core Data 오버라이드
  - `prepareToEnd`: 창 닫을 때 Rust SessionManager에 저장 (dual-write)
  - `saveSessionToRust`: scrollPosition NSData → ScSessionState 변환 포함
- `SessionManager::exists()` 추가 (sc-storage)

**Dual-write 전략**: Core Data + Rust 양쪽 동시 유지. 다음 스프린트에서 Core Data 제거.

### Sprint 15 — Phase 6: 스크롤 복원 + 세션 삭제 + Rust 썸네일 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass (변화 없음) |
| clippy | 경고 0 |
| 커밋 | 1006fa8 |

**배선 완료 (5/6 Phase 6):**
- 세션 스크롤 복원: Rust scroll_x/y → NSKeyedArchived NSData → `session.scrollPosition`
- 세션 삭제 동기화: `endSession:` → `sc_session_delete_c` (Core Data + Rust 동시 삭제)
- `sc_thumbnail_from_bytes` C FFI: Rust Lanczos3 스케일링 → RGBA bytes
- `TSSTPage.prepThumbnail` → Rust 우선, AppKit 폴백 (GIF 등)
- 세션 Phase 6 완성: 저장/복원/삭제 모두 Rust 동기화

**Phase 6 잔여:**
- `TSSTPageView.m` — 이미지 렌더링 → Rust 파이프라인 (스케일+컴포지팅)

### Sprint 16 — Phase 6 완료: TSSTPage.pageImage Rust 사전 스케일링 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass (sc-ffi: 22 pass, +2 신규) |
| clippy | 경고 0 |
| 커밋 | 4d45c88 |

**Phase 6 UI 배선 6/6 완료:**
- `sc_cap_image_bytes` C FFI: max(w,h) > 2048px → Rust Lanczos3 사전 스케일링
- `TSSTPage.pageImage`: 대형 이미지 메모리 최적화 (2048px 캡)
  - CALayer GPU 렌더링 유지 (품질 무손실)
  - AppKit 폴백: animated GIF / Rust 디코드 실패 시
- `kSCMaxInMemoryDimension = 2048` 상수

**Phase 6 완료 요약 (Sprint 11-16):**
| Sprint | 배선 대상 |
|--------|---------|
| 11 | AppDelegate 아카이브 감지 → Rust |
| 12 | requestDataForPageIndex → Rust |
| 13 | nestedArchiveContents 열거 → Rust ScPageList |
| 14 | TSSTSessionWindowController 세션 CRUD → Rust |
| 15 | 스크롤 복원 + 세션 삭제 + 썸네일 Rust |
| 16 | pageImage 대형 이미지 메모리 최적화 |

**다음: Phase 7 OCR 통합 또는 Phase 9 최종 검증**

### Sprint 17 — Phase 9 시작: 정리 + 벤치마크 검증 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass |
| clippy | 경고 0 |
| 커밋 | 7aa984e + 61613b4 |

**완료:**
- `nestedFolderContents` archiveExtensions → `sc_archive_is_supported` (마지막 archive detection 교체)
- `TSSTPage.m` XADMaster import 제거
- CHANGELOG.md 작성 (Phase 6 전체)
- 생성된 Swift 바인딩 파일 git 추가
- 벤치마크 `.cbz` suffix 버그 수정

**Phase 9 벤치마크 기준값 (2026-06-02):**

| 측정 항목 | 시간 |
|---------|------|
| natural_sort 100개 파일명 | 424 µs |
| is_image_entry 1000x | 50 µs |
| cbz_open_and_list_50pages | 458 µs |
| cbz_read_first_image_50pages | 682 µs |
| image scale fit_window 800×1200→1024×768 | 19.7 ms |
| two_page_spread compositor | 6.4 ms |
| thumbnail_parallel 10 entries | 1.1 ms |
| thumbnail_parallel 50 entries | 4.1 ms |
| thumbnail_serial 50 entries | 20.5 ms (×5 serial vs parallel) |

**잔여 XADMaster 의존성:**
- `nestedArchiveContents` Phase 2: 중첩 아카이브 + PDF 추출 (edge case)
- `TSSTManagedArchive.instance`: XADArchive 핸들 (Phase 2 전용)
- `EightViewsAppDelegate.archiveTypes`: NSOpenPanel 파일 타입 필터 (UI only)

### Sprint 18 — Phase 9: 긴급 색상 버그 수정 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 23 pass (sc-ffi, +1 신규) |
| clippy | 경고 0 |
| 커밋 | 9705dd5 |

**긴급 수정 (Sprint 15/16 도입 버그):**
- `NSBitmapFormatAlphaFirst` (ARGB = 알파 첫째) → `NSBitmapFormatAlphaNonpremultiplied` (RGBA = 알파 마지막)
- Rust `image::to_rgba8().into_raw()` = RGBA 순서 (알파 마지막)
- 잘못된 포맷 사용 시: 빨강/알파 채널 뒤바뀜 → 썸네일과 대형 이미지 색상 오류
- `thumbnail_pixel_order_is_rgba` 회귀 테스트 추가 (bytes[3]=255 확인)

### Sprint 19 — Phase 9: 색상 공간 수정 + README + Push (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass |
| clippy | 경고 0 |
| 커밋 | 90e8d2a |

**완료:**
- `NSDeviceRGBColorSpace` → `NSCalibratedRGBColorSpace` (sRGB 색상 관리 적용)
  - 만화 이미지는 sRGB 색 공간; P3 디스플레이에서 올바른 색상 매핑
  - prepThumbnail + pageImage 두 곳
- README.md: arc 브랜치 Rust 코어 빌드 지침 추가
- Sprint Completion Gate: 41커밋 → `origin/arc` push 시도
  - HTTPS 인증 필요 → `git push origin arc` 수동 실행 필요

### Sprint 20 — Phase 8 시작: QuickLook Rust 부분 통합 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass |
| clippy | 경고 0 |
| 커밋 | c5e07e8 |

**완료:**
- `sc_archive_read_first_image` C FFI: 첫 이미지만 읽는 최적화 경로
- `GenerateThumbnailForURL.m`: XADArchive → `sc_archive_read_first_image`
  - 덮개 이름 없는 경우(else 브랜치) Rust 교체
  - DTPartialArchiveParser(덮개 이름 있는 경우) 유지
- `libeightviews.a` universal 재빌드

**Sprint Completion Gate: 스프린트 10개 연속 한계 도달 (Sprint 11-20)**
**⚠ 다음 스프린트 실행 전 반드시 GitHub push 필요 ⚠**
```
git remote set-url origin git@github.com:MaddTheSane/Simple-Comic.git
git push origin arc
```

### Sprint 21 — Phase 8: QuickLook 미리보기 Rust 완성 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass |
| clippy | 경고 0 |
| 커밋 | 394bbd1 |

**완료:**
- `GeneratePreviewForURL.m`: XADArchive 열거 + `fileListForArchive` + `fileSort` 제거
  - `sc_archive_open_pages` → 자연 정렬 페이지 목록
  - `sc_archive_read_page` → 페이지별 이미지 데이터
  - 시간 제한(~1초), 취소 감지, CGImageSource 폴백 유지

**Phase 8 QuickLook 현황:**
- ✅ `GenerateThumbnailForURL.m` (구형 ObjC API) — Rust partial-read (Sprint 20)
- ✅ `GeneratePreviewForURL.m` (구형 ObjC API) — Rust 페이지네이션 (Sprint 21)
- ✅ `ThumbnailProvider.swift` (신형 Swift API) — Rust C FFI (Sprint 22)
- ✅ `PreviewProvider.swift` (신형 Swift API) — Rust C FFI (Sprint 22)

**Phase 8 완료.**

### Sprint 22 — Phase 8 완료: Swift QuickLook 신형 API Rust 교체 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 전체 pass (변화 없음) |
| clippy | 경고 0 |
| 커밋 | 11f5e63 (cleanup) + 15eb446 (Sprint 22) |

**완료:**
- `ThumbnailProvider.swift`: XADArchive 제거
  - coverName 없는 경우 → `sc_archive_read_first_image` (최적화 경로)
  - coverName 있는 경우 → `sc_archive_open_pages` + 이름 매칭 + `sc_archive_read_page`
- `PreviewProvider.swift`: XADArchive 제거
  - `sc_archive_open_pages` → count 획득 → `sc_archive_read_page` 루프
  - 원본 페이지 선택 로직 (index >= 25 || last) 보존
- `PartialArchiveParser.swift`, `QuickComicCommonSwift.swift`: XADMaster 의존성 제거 (빈 클래스로 교체)
- `project.pbxproj`: 6개 빌드 설정에 HEADER_SEARCH_PATHS / LIBRARY_SEARCH_PATHS / OTHER_LDFLAGS 추가
  - 대상: 메인 앱 Debug/Release, QuickComic Thumbnailer Debug/Release, QuickComic Preview Debug/Release

**Sprint 23 후보 (리뷰에서 도출):**
- **HIGH**: `sc_archive_read_page` N+1 archive 재오픈 성능 — QLPreviewReply 클로저에서 count회 반복 시 지연
  → `sc_archive_open_once` 스타일 핸들 캐시 API 추가 검토
- Phase 7 OCR 통합 또는 Phase 9 최종 검증 마무리

### Sprint 23 — PreviewProvider 정리 + Phase 9 Sprint 27 벤치마크 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 116 pass / 0 fail |
| clippy | 경고 0 |
| 커밋 | c46679b + a4265f9 |

**사전 정리 (PreviewProvider):**
- `index >= 25` → `index < 25` 버그 수정 (처음 25페이지 + 마지막 페이지 표시)
- Pre-fetch 패턴: `providePreview` async body에서 페이지 미리 읽기 → 클로저 내 N+1 아카이브 재오픈 제거

**Phase 9 Sprint 27 — 200페이지 벤치마크 검증:**

| 지표 | PLAN.md 목표 | 실측값 | 판정 |
|------|-------------|--------|------|
| 아카이브 오픈 (200p CBZ) | < 500 ms | 1.13 ms | ✅ 443× 여유 |
| 페이지 전환 (read_page 200p) | < 50 ms | 2.01 ms | ✅ 25× 여유 |
| QuickLook 썸네일 (first image 200p) | < 1 s | 1.72 ms | ✅ 580× 여유 |
| 썸네일 병렬 (200p) | < 3 s | 25.4 ms | ✅ 118× 여유 |
| 메모리 (200p CBZ) | < 200 MB | ⏳ Instruments 필요 | 미측정 |

**잔여 Phase 9 작업 (Sprint 28):**
- 메모리 검증: Instruments Leaks/Allocations — Xcode에서 직접 실행 필요
- Core Data 의존성 제거: `Sessions_DataModel.xcdatamodeld` 삭제 (Phase 4 마이그레이션 완료 후)
- CHANGELOG v2.0.0 + 태그 생성

**ObjC 데드 코드 감사 결과:**
- DTPartialArchiveParser, TSSTSortDescriptor, TSSTImageUtilities, UKXattrMetadataStore — 모두 여전히 사용 중 (제거 불가)
- 잔여 XADMaster 의존성: TSSTManagedGroup.m (중첩 아카이브/고체 RAR), TSSTSessionWindowController.m (cover name lookup), EightViewsAppDelegate.m (archiveTypes) — Phase 2 edge case, 의도적 유지

### Sprint 24 — Phase 9 Sprint 28: v2.0.0-alpha 릴리즈 준비 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 116 pass / 0 fail |
| clippy | 경고 0 |
| 커밋 | d8e321a |
| 태그 | v2.0.0-alpha |

**완료:**
- `Cargo.toml` workspace version: 0.1.0 → 0.2.0-alpha
- `sc_version()` C FFI 버전 문자열 갱신
- `CHANGELOG.md` 완성: Phase 7 (deferred) + Phase 8 + Phase 9 + Known Limitations
- `README.md` arc 브랜치 상태 표 업데이트
- `git tag v2.0.0-alpha` 생성

**v2.0.0-alpha Known Limitations:**
- Core Data 세션 마이그레이션 도구 없음 (Phase 4 미완)
- OCR 검색 인덱스 미연결 (Phase 7 deferred)
- Instruments 메모리 검증 미실시

**Phase 9 완료 (Sprint 28 기준):**
- ✅ 벤치마크 — 모든 목표 달성 (Sprint 23)
- ✅ CHANGELOG + 버전 태그 (Sprint 24)
- ⏳ Instruments Leaks 검증 — Xcode에서 사용자 직접 실행
- ⏳ Core Data 제거 — Phase 4 완성 후 Sprint 25에서 재검토

### Sprint 26 — Phase 4: Core Data → Rust SQLite 세션 마이그레이션 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 119 pass / 0 fail (+3 신규) |
| clippy | 경고 0 |
| 커밋 | 631e5f3 |

**완료:**
- `EightViewsAppDelegate.m`: `migrateCoreDateSessionsToRust` 메서드
  - `applicationDidFinishLaunching:` → `sessionRelaunch` 직전에 호출
  - `NSUserDefaults@SCRustSessionMigrationDone` 플래그로 1회만 실행 (멱등)
  - `Core Data session` → `fileURL.path` → `sc_session_exists` 체크 → `sc_session_save`
  - `scrollPosition` NSKeyedArchive NSPoint 디코딩 포함
- `sc-ffi/src/lib.rs`: `sc_session_exists()` C FFI 추가
  - 상태 역직렬화 없이 레코드 존재 여부만 확인 (sc_session_load 대비 효율적)
- `sc_extras.h`: `sc_session_exists` 선언 추가
- 3개 단위 테스트 추가 (false-before-save, true-after-save, null-path)

**Phase 4 완료 현황:**
- ✅ SessionManager SQLite CRUD (sc-storage, Phase 5-6에서 구현)
- ✅ dual-write (Core Data + Rust 동시 저장, Phase 6)
- ✅ **기존 세션 마이그레이션 (이번 Sprint 26)**
- ⏳ Core Data 의존성 완전 제거 (groups/hierarchy 스키마 완성 후)

### Sprint 28 — Phase 7: OCR 텍스트 캐시 + 2단계 검색 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 123 pass / 0 fail (+4 신규: OcrCache) |
| clippy | 경고 0 |
| 커밋 | edc2c8a |

**완료:**
- `sc-storage`: SCHEMA_V3 `ocr_cache` 테이블 + `OcrCache` struct (store/get/has/clear) + 4 테스트
- `sc-ffi`: `sc_ocr_store`, `sc_ocr_has`, `sc_ocr_get`, `sc_ocr_clear` C FFI
- `sc_extras.h`: OCR cache 선언 추가
- `OCRFind.h`: `OCRFindDelegate`에 `rustSessionArchivePath` 프로퍼티 추가
- `OCRFind.m`: 2단계 검색 구현
  - 빠른 경로: Rust 캐시 → 텍스트 검색 → 매치 시 Vision(렌더링만)
  - 느린 경로: Vision OCR → Rust 캐시 저장 → 검색 (최초 페이지)
  - 재검색 시 Vision 생략 → 대폭 빠름
- `TSSTSessionWindowController.m`: `sc_ocr_clear` 세션 종료 시 호출

**Phase 7 완료 현황:**
- ✅ OCR 텍스트 캐시 (Rust SQLite)
- ✅ 2단계 검색 (캐시 → Vision)
- ⏳ Phase 7 원래 계획 중 `OCRFind.m` 검색 로직 완전 Rust 교체 — 현재 검색은 ObjC NSString, Vision 결과 rendering은 여전히 ObjC. Rust `aho-corasick` 완전 교체는 별도 스프린트

### Sprint 29 — Phase 7: OCR 캐시 영속화 + mtime 스태일니스 (2026-06-02)

| 항목 | 결과 |
|------|------|
| Rust tests | 124 pass / 0 fail (+1: has_valid_matches_mtime) |
| clippy | 경고 0 |
| 커밋 | 7108053 |

**완료:**
- `SCHEMA_V4`: `ALTER TABLE ocr_cache ADD COLUMN archive_mtime` (멱등)
- `OcrCache::store(mtime)` + `OcrCache::has_valid(path, idx, mtime)` — mtime 불일치 시 false
- C FFI: `sc_ocr_store` 4번째 인자 mtime 추가 / `sc_ocr_has_valid` 신규
- OCRFind.m: `NSFileManager`로 아카이브 mtime 획득 → `sc_ocr_has_valid` 사용
- `TSSTSessionWindowController.m`: `sc_ocr_clear` 세션 종료 호출 **제거**
  - 캐시가 세션 간 영속됨 (Vision OCR이 아카이브당 페이지당 1회만 실행)
  - 아카이브 mtime 변경 시 자동 무효화

**Phase 7 최종 현황:**
- ✅ OCR 텍스트 캐시 (Sprint 28)
- ✅ 2단계 검색 (Sprint 28)
- ✅ 캐시 영속화 + mtime 스태일니스 (Sprint 29)
- ⏳ aho-corasick 완전 교체 (선택적)

**⚠ 63 커밋 미push — GitHub 포크 필요 ⚠**
```bash
# 자신의 포크에 push:
git remote add mine git@github.com:hephaex/Simple-Comic.git
git push mine arc
git push mine v2.0.0-alpha
```

---

*최종 업데이트: 2026-06-02*
