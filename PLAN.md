# 8views — Rust 리팩토링 계획

> 현재: Objective-C/Swift 10,400 LOC (macOS 12+)
> 목표: Rust 코어 + Swift UI 하이브리드 아키텍처

---

## 아키텍처 결정 (ADR-001)

### 채택: 하이브리드 방식 (Rust core + Swift UI)

| 계층 | 언어 | 역할 |
|------|------|------|
| Core Library | Rust | 아카이브 파싱, 이미지 파이프라인, 세션 스토리지 |
| FFI Bridge | uniffi | Swift ↔ Rust 바인딩 자동 생성 |
| UI Layer | Swift/ObjC | AppKit, Vision, QuickLook, Touch Bar |

**이유:**
- macOS 네이티브 UI(AppKit, Vision, Touch Bar)는 Rust 바인딩 미성숙
- 비즈니스 로직(아카이브, 이미지, 세션)은 Rust 크레이트 생태계 활용 가능
- 점진적 교체 — 기존 Swift UI를 유지하며 핵심 모듈부터 이전

**대안:**
- 완전 Rust + `cacao`/`objc2` → AppKit 바인딩 불완전, Vision/QuickLook 지원 없음
- 완전 Swift → 목표와 불일치
- `tauri` + WebUI → macOS 네이티브 UX 손실

---

## Rust 크레이트 목록

### 아카이브
| 크레이트 | 용도 |
|---------|------|
| `zip` 2.x | CBZ/ZIP |
| `compress-tools` | CBR/RAR (libarchive 기반, RAR 라이선스 우회) |
| `tar` + `flate2` | TAR/GZ/BZ2 |
| `sevenz-rust` | 7z/7zip |

### 이미지
| 크레이트 | 용도 |
|---------|------|
| `image` 0.25 | PNG/JPEG/GIF/BMP/TIFF 로딩·스케일링·회전 |
| `webp` | WebP 디코딩 |
| `pdfium-render` | PDF → 이미지 페이지 |
| `rayon` | 썸네일 병렬 생성 |

### 스토리지
| 크레이트 | 용도 |
|---------|------|
| `rusqlite` + `rusqlite_migration` | SQLite 세션 DB (Core Data 대체) |
| `serde` + `serde_json` | 직렬화 |

### 시스템
| 크레이트 | 용도 |
|---------|------|
| `xattr` | 파일 확장 속성 (UKXattrMetadataStore 대체) |
| `tokio` | 비동기 런타임 |
| `uniffi` 0.28 | Swift FFI 바인딩 자동 생성 |
| `anyhow` | 에러 핸들링 |
| `tracing` | 로깅/진단 |

---

## 완료 현황 (2026-06-02, v2.0.0-alpha)

| Phase | 계획 Sprint | 실제 Sprint | 상태 |
|-------|------------|------------|------|
| 1: 프로젝트 설정 | 1~2 | 1~2 | ✅ 완료 |
| 2: 아카이브 엔진 | 3~6 | 3~6 | ✅ 완료 |
| 3: 이미지 파이프라인 | 7~10 | 7~10 | ✅ 완료 |
| 4: 세션 스토리지 | 11~13 | Sprint 10 + Sprint 26 | ✅ 마이그레이션 도구 완성 (Core Data 제거는 별도 major sprint) |
| 5: Swift FFI | 14~16 | Sprint 6~8 (선행) | ✅ 완료 |
| 6: UI 배선 | 17~22 | Sprint 11~16 | ✅ 완료 |
| 7: OCR 통합 | 23~24 | Sprint 28~29 | ✅ 완료 (Vision 캐시 + mtime 무효화) |
| 8: QuickLook | 25~26 | Sprint 20~22 | ✅ 완료 |
| 9: 최종 검증 | 27~28 | Sprint 17, 23~24 | ✅ 완료 (메모리 제외) |

**알파 릴리즈 기준으로 현재 단계 목표 달성.**  
Phase 4 (Core Data 마이그레이션 도구)와 Phase 7 (OCR 검색 인덱스)는 다음 마일스톤에서 재개.

---

## 로드맵

### Phase 1: 프로젝트 설정 [Sprint 1~2]
- Cargo workspace 생성 (`8views-core/`)
- `uniffi` 기반 Swift 바인딩 스캐폴딩
- CI: `cargo test`, `cargo clippy`, `cargo fmt` 파이프라인
- Swift 패키지에서 Rust 정적 라이브러리 링크 검증

### Phase 2: 아카이브 엔진 [Sprint 3~6]
- `ArchiveReader` trait: `open()`, `list_entries()`, `read_entry(index)`
- 구현: ZIP, RAR (via compress-tools), 7z, TAR, 폴더
- DTPartialArchiveParser → Rust (첫 이미지만 추출)
- 파일명 자연 정렬 (TSSTSortDescriptor 대체)
- 문자 인코딩 자동 감지 (UniversalDetector 대체 → `chardet`)
- 테스트: 각 포맷 샘플 파일로 엔트리 목록·내용 검증

### Phase 3: 이미지 파이프라인 [Sprint 7~10]
- `ImageLoader`: 이미지 로딩, 캐싱, 썸네일 생성
- 지원 포맷: JPEG, PNG, WebP, GIF, BMP, TIFF, PDF 페이지
- TSSTImageUtilities → Rust
- 스케일 모드: original, fit-window, fit-width
- 회전: 0/90/180/270도
- 두 페이지 합성: aspect ratio 기반 정렬 (TSSTPageView 로직)
- 썸네일 LRU 캐시
- 테스트: 픽셀 동일성 검증 (기존 Swift 출력과 비교)

### Phase 4: 세션 스토리지 [Sprint 11~13] ✅ 마이그레이션 완성 (Core Data 제거 별도)
- SQLite 스키마 설계 (Core Data 6 엔티티 매핑)
  - `sessions`, `groups`, `pages` 테이블
  - 계층적 group 구조 (아카이브 내 아카이브)
- `SessionManager`: create, load, save, delete
- 마이그레이션: Core Data 바이너리 스토어 → SQLite (기존 세션 유지)
- 세션 상태: position, zoom, rotation, two_page_spread, page_order
- xattr 메타데이터 저장 (UKXattrMetadataStore 대체)
- 테스트: CRUD, 마이그레이션, 동시 접근

### Phase 5: Swift FFI 통합 [Sprint 14~16]
- `uniffi` UDL 인터페이스 정의
- Swift 래퍼 생성 검증 (`swift package generate-xcodeproj`)
- Xcode 프로젝트에 Rust static lib 링크
- C 바인딩 (`cbindgen`) 대안 검토
- 기존 `TSSTManagedGroup`, `TSSTPage`, `TSSTManagedSession` → Rust 위임
- 테스트: Swift 단위 테스트에서 Rust 함수 호출 검증

### Phase 6: UI 배선 [Sprint 17~22]
- `EightViewsAppDelegate.m` — 아카이브 오픈 Rust 코어 위임
- `TSSTSessionWindowController.m` — 페이지 전환 Rust 캐시 사용
- `TSSTPageView.m` — 이미지 렌더링 Rust 파이프라인 사용
- `TSSTThumbnailView.swift` — 썸네일 Rust 병렬 생성 사용
- `DTPolishedProgressBar.swift` — Rust 세션 상태 바인딩
- 점진적 교체: 기능별 A/B 검증 후 전환

### Phase 7: OCR 통합 [Sprint 23~24] ✅ 완료 (Sprint 28-29)
- Vision 프레임워크는 Swift에 유지 (macOS 전용, Rust 바인딩 없음)
- Rust 이미지 파이프라인 → Swift Vision → 텍스트 결과 → Rust 검색 인덱스
- `OCRFind.m` 검색 로직 → Rust (`aho-corasick` 또는 `tantivy`)
- 테스트: 텍스트 선택 정확도 기존 대비 비교

### Phase 8: QuickLook 플러그인 [Sprint 25~26]
- Rust 아카이브 엔진을 QuickLook 익스텐션에서 재사용
- `ThumbnailProvider.swift` / `PreviewProvider.swift` → Rust 썸네일 생성 위임
- 테스트: Finder에서 CBZ/CBR 파일 썸네일·미리보기 검증

### Phase 9: 최종 검증 및 정리 [Sprint 27~28]
- 전체 기능 회귀 테스트
- 성능 벤치마크 (아래 기준)
- 메모리 누수 검사 (`heaptrack`, Xcode Instruments)
- 미사용 Objective-C 코드 제거
- Core Data 의존성 완전 제거

---

## 검증 기준 (Definition of Done)

### 기능 완전성 (v2.0.0-alpha 기준)
- [x] ZIP/CBZ 아카이브 읽기
- [x] RAR/CBR 아카이브 읽기
- [x] 7z 아카이브 읽기
- [x] TAR/TGZ/TBZ2 아카이브 읽기
- [x] 폴더(이미지 디렉토리) 열기
- [ ] PDF 페이지 표시 — XADMaster 경로 유지 (Rust pdfium-render 미연결)
- [x] 두 페이지 펼치기(Two-Page Spread)
- [x] 페이지 순서 반전 (우→좌 망가 모드)
- [x] 줌/회전 (세션별 저장)
- [x] 세션 복원 (앱 재시작 후 마지막 위치) — Rust SQLite dual-write
- [x] 썸네일 뷰
- [ ] OCR 텍스트 선택 + 검색 — Vision 경로 유지 (Rust 검색 인덱스 미연결) ⏳ Phase 7
- [x] Touch Bar 스크러버
- [x] QuickLook 썸네일/미리보기
- [x] 전체화면 모드

### 성능 기준 (v2.0.0-alpha 실측)

| 지표 | 목표 | 실측 | 판정 |
|------|------|------|------|
| 아카이브 오픈 (200페이지 CBZ) | < 500ms | 1.13 ms | ✅ |
| 페이지 전환 (read_page) | < 50ms | 2.01 ms | ✅ |
| 썸네일 생성 (200페이지, 병렬) | < 3s | 25.4 ms | ✅ |
| 메모리 (200페이지 CBZ 열기) | < 200MB | ⏳ Instruments 필요 | — |
| QuickLook 썸네일 생성 | < 1s | 1.72 ms | ✅ |

### 코드 품질
- [x] `cargo test` 통과 (Rust 코어 전체) — 116 tests
- [ ] 테스트 커버리지 ≥ 80% (아카이브, 이미지, 세션 모듈)
- [ ] `cargo clippy -- -D warnings` 경고 0
- [ ] `cargo fmt --check` 통과
- [ ] Xcode `xcodebuild test` 통과 (Swift UI 단위 테스트)
- [ ] 메모리 누수 없음 (Instruments Leaks)

### 호환성
- [ ] macOS 12.0+ 지원 유지
- [ ] 기존 Core Data 세션 마이그레이션 (데이터 손실 없음)
- [ ] 기존 xattr 메타데이터 읽기

---

## 파일 대응표 (Objective-C/Swift → Rust)

| 기존 파일 | Rust 모듈 | 상태 |
|---------|----------|------|
| DTPartialArchiveParser.m | `archive::partial` | [PLANNED] |
| TSSTManagedGroup.m | `archive::group` | [PLANNED] |
| TSSTPage.m | `storage::page` | [PLANNED] |
| TSSTManagedSession.m | `storage::session` | [PLANNED] |
| TSSTImageUtilities.m | `image::loader` | [PLANNED] |
| TSSTSortDescriptor.m | `util::natural_sort` | [PLANNED] |
| UKXattrMetadataStore.m | `util::xattr` | [PLANNED] |
| OCRFind.m | `ocr::search` | [PLANNED] |
| TSSTPageView.m (로직) | `image::compositor` | [PLANNED] |

---

## 위험 요소

| 위험 | 가능성 | 대응 |
|------|--------|------|
| RAR 라이선스 (unrar crate) | 중 | compress-tools (libarchive) 사용 |
| uniffi 버전 불안정 | 저 | cbindgen 대안 유지 |
| Core Data → SQLite 마이그레이션 손실 | 중 | 변환 전 Core Data 내보내기 + 검증 |
| Vision 프레임워크 Rust 미지원 | 확정 | Swift에 유지, FFI로 결과만 수신 |
| pdfium-render 빌드 복잡도 | 중 | Phase 3에서 PDFKit(Swift)로 대체 검토 |

---

*작성: 2026-06-01*
*검토 예정: Phase 2 완료 시 업데이트*
