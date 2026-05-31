# Simple Comic — Rust 리팩토링 진행 상황

> 시작: 2026-06-01
> 현재 Phase: 2 (Sprint 4 완료)

---

## 전체 진행률

```
Phase 1: 설정          [x] 2/2 sprint (완료)
Phase 2: 아카이브 엔진  [~] 2/4 sprint (Sprint 4 완료)
Phase 3: 이미지 파이프라인 [ ] 0/4 sprint
Phase 4: 세션 스토리지   [ ] 0/3 sprint
Phase 5: Swift FFI     [ ] 0/3 sprint
Phase 6: UI 배선        [ ] 0/6 sprint
Phase 7: OCR 통합       [ ] 0/2 sprint
Phase 8: QuickLook     [ ] 0/2 sprint
Phase 9: 최종 검증      [ ] 0/2 sprint
────────────────────────────────────
합계: 0/28 sprint
```

---

## Phase 1: 프로젝트 설정

### Sprint 1: Cargo Workspace 구성 ✅ (2026-06-01)
- [x] `simple-comic-core/` Cargo workspace 생성
- [x] 크레이트 구조 설계: `sc-core`, `sc-archive`, `sc-image`, `sc-storage`, `sc-ffi`
- [x] uniffi 0.29 + zip/sevenz/tar/image/rusqlite 의존성 설정
- [x] CI 설정: GitHub Actions `cargo test`, `cargo clippy`, `cargo fmt`
- [x] `cargo check/test/clippy/fmt` 전체 통과 (12 tests)
- [x] 커밋: 8833a8d

### Sprint 2: 개발 환경 & 벤치마크 기준선 ✅ (2026-06-01)
- [x] 크로스 컴파일 검증: arm64 + x86_64 → `libsimplecomic-universal.a` 11MB
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

### Sprint 5: 7z + 폴더 지원
- [ ] `SevenZipArchive` 구현 (`sevenz-rust` 크레이트)
- [ ] `FolderReader` 구현 (이미지 파일만 필터링)
- [ ] `PartialReader`: 첫 이미지만 추출 (QuickLook용)
- [ ] 단위 테스트: 7z, 폴더 열기

### Sprint 6: 아카이브 엔진 통합 + FFI 노출
- [ ] `ArchiveFactory`: 파일 확장자로 적절한 구현 선택
- [ ] uniffi UDL: `open_archive()`, `list_pages()`, `read_page_data()` 노출
- [ ] 통합 테스트: 모든 포맷 end-to-end
- [ ] 성능 벤치마크: 200페이지 CBZ 오픈 < 500ms

---

## Phase 3: 이미지 파이프라인

### Sprint 7: 기본 이미지 로딩
- [ ] `ImageLoader`: JPEG/PNG/GIF/BMP/TIFF 로딩
- [ ] WebP 지원 (`webp` 크레이트)
- [ ] 이미지 메타데이터: 너비, 높이, aspect ratio
- [ ] LRU 캐시 (`lru` 크레이트, 최대 50개 페이지)
- [ ] 단위 테스트: 각 포맷 로딩, 캐시 히트/미스

### Sprint 8: 스케일링 + 회전
- [ ] 스케일 모드: original, fit-window, fit-width
- [ ] 회전: 0/90/180/270도
- [ ] 고품질 다운샘플링 (Lanczos/Mitchell)
- [ ] 단위 테스트: 각 모드 출력 크기 검증

### Sprint 9: 두 페이지 합성 + 썸네일
- [ ] `Compositor`: 두 이미지 side-by-side 합성
- [ ] aspect ratio 기반 정렬 (TSSTPageView 로직 이식)
- [ ] 썸네일 생성: rayon 병렬 처리
- [ ] 단위 테스트: 합성 결과 픽셀 비교 (기존 출력 대비)

### Sprint 10: PDF 지원 + 파이프라인 통합
- [ ] PDF 페이지 → 이미지 (PDFKit Swift 래퍼 or pdfium-render)
- [ ] uniffi UDL: `load_image()`, `get_thumbnail()`, `composite_two_pages()` 노출
- [ ] 성능 벤치마크: 썸네일 200개 생성 < 3s

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
- [ ] UDL 파일 전체 작성 (`simplecomic.udl`)
- [ ] `cargo build` → `libsimplecomic.a` + Swift 바인딩 파일 자동 생성
- [ ] Swift Package에서 빌드 검증

### Sprint 15: Xcode 통합
- [ ] Xcode 프로젝트 링크 설정 (`-lsimplecomic`, `-lc++`)
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
- [ ] `SimpleComicAppDelegate.m` — 아카이브 오픈 → Rust `open_archive()`
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

---

*최종 업데이트: 2026-06-01*
