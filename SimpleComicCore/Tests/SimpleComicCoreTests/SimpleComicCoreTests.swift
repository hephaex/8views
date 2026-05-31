import XCTest
@testable import SimpleComicCore

final class SimpleComicCoreTests: XCTestCase {
    func testLibraryVersion() {
        let version = scLibraryVersion()
        XCTAssertFalse(version.isEmpty)
        // SemVer: MAJOR.MINOR.PATCH
        XCTAssertEqual(version.split(separator: ".").count, 3)
    }

    func testArchiveListMissingReturnsError() {
        XCTAssertThrowsError(try archiveListPages(archivePath: "/nonexistent/file.cbz")) { error in
            XCTAssertTrue(error is ScError)
        }
    }

    func testSessionLoadMissingReturnsDefault() throws {
        let state = try sessionLoad(archivePath: "/tmp/nonexistent.cbz")
        XCTAssertEqual(state.pageIndex, 0)
        XCTAssertFalse(state.twoPageSpread)
        XCTAssertEqual(state.zoomLevel, 1.0, accuracy: 0.001)
    }

    func testSessionSaveAndLoad() throws {
        let path = "/tmp/test_session_\(ProcessInfo.processInfo.globallyUniqueString).cbz"
        let state = SessionStateRecord(
            pageIndex: 42,
            zoomLevel: 1.5,
            rotationDegrees: 90,
            twoPageSpread: true,
            rightToLeft: true,
            scaleMode: 2,
            scrollX: 10.0,
            scrollY: 20.0
        )
        try sessionSave(archivePath: path, state: state)
        let loaded = try sessionLoad(archivePath: path)
        XCTAssertEqual(loaded.pageIndex, 42)
        XCTAssertEqual(loaded.zoomLevel, 1.5, accuracy: 0.001)
        XCTAssertEqual(loaded.rotationDegrees, 90)
        XCTAssertTrue(loaded.twoPageSpread)
    }
}
