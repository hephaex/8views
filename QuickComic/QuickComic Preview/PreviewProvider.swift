//
//  PreviewProvider.swift
//  QuickComic Preview
//
//  Created by C.W. Betts on 12/11/22.
//  Copyright © 2022 Dancing Tortoise Software. All rights reserved.
//

import Cocoa
import Quartz
import UniformTypeIdentifiers

private func newPDFPage(from image: NSImage) -> PDFPage? {
	if #available(macOSApplicationExtension 13.0, *) {
		return PDFPage(image: image, options: [.compressionQuality: 0.5])
	} else {
		return PDFPage(image: image)
	}
}

class PreviewProvider: QLPreviewProvider, QLPreviewingController {

    func providePreview(for request: QLFilePreviewRequest) async throws -> QLPreviewReply {
        let archivePath = request.fileURL.path
        var errCode: Int32 = 0
        guard let pageList = sc_archive_open_pages(archivePath, &errCode) else {
            throw CocoaError(.fileReadCorruptFile, userInfo: [
                NSURLErrorKey: request.fileURL,
                NSLocalizedDescriptionKey: NSLocalizedString("No images found in archive.", comment: "No images found in archive."),
            ])
        }
        let count = Int(sc_page_list_count(pageList))
        sc_archive_pages_free(pageList)

        guard count > 0 else {
            throw CocoaError(.fileReadCorruptFile, userInfo: [
                NSURLErrorKey: request.fileURL,
                NSLocalizedDescriptionKey: NSLocalizedString("No images found in archive.", comment: "No images found in archive."),
            ])
        }

        // Pre-fetch the pages we need (first 25 + last) before the QLPreviewReply closure.
        // This avoids N+1 archive re-opens inside the closure (sc_archive_read_page is
        // path-keyed and opens the archive anew each call).
        var prefetchedPages = [Data?]()
        prefetchedPages.reserveCapacity(min(count, 26))

        for index in 0..<count {
            // Show the first 25 pages + last page (mirrors original intent).
            guard index < 25 || index == count - 1 else { continue }
            var pageLen = 0
            var pageErr: Int32 = 0
            if let ptr = sc_archive_read_page(archivePath, UInt32(index), &pageLen, &pageErr) {
                let data = Data(bytes: ptr, count: pageLen)
                sc_free_bytes(ptr, pageLen)
                prefetchedPages.append(data)
            } else {
                prefetchedPages.append(nil)
            }
        }

        // Determine PDF page size from the first successfully pre-fetched image.
        let pdfSize: CGSize = prefetchedPages
            .compactMap { $0.flatMap { NSImage(data: $0)?.size } }
            .first ?? CGSize(width: 800, height: 600)

        let reply = QLPreviewReply(forPDFWithPageSize: pdfSize) { _ in
            let document = PDFDocument()
            for pageData in prefetchedPages {
                guard let pageData,
                      let image = NSImage(data: pageData),
                      let page = newPDFPage(from: image) else {
                    let badPage = PDFPage()
                    badPage.setBounds(NSRect(origin: .zero, size: pdfSize), for: .mediaBox)
                    document.insert(badPage, at: document.pageCount)
                    continue
                }
                document.insert(page, at: document.pageCount)
            }
            return document
        }

        return reply
    }
}
