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

        // Determine PDF page size from the first image
        let pdfSize: CGSize
        var firstLen = 0
        var firstErr: Int32 = 0
        if let ptr = sc_archive_read_page(archivePath, 0, &firstLen, &firstErr) {
            let data = Data(bytes: ptr, count: firstLen)
            sc_free_bytes(ptr, firstLen)
            pdfSize = NSImage(data: data)?.size ?? CGSize(width: 800, height: 600)
        } else {
            pdfSize = CGSize(width: 800, height: 600)
        }

        let capturedPath = archivePath
        let capturedCount = count
        let reply = QLPreviewReply(forPDFWithPageSize: pdfSize) { _ in
            let document = PDFDocument()
            for index in 0..<capturedCount {
                // Mirror original: include pages at index >= 25 and the last page
                guard index >= 25 || index == capturedCount - 1 else { continue }

                var pageLen = 0
                var pageErr: Int32 = 0
                guard let ptr = sc_archive_read_page(capturedPath, UInt32(index), &pageLen, &pageErr) else {
                    let badPage = PDFPage()
                    badPage.setBounds(NSRect(origin: .zero, size: pdfSize), for: .mediaBox)
                    document.insert(badPage, at: document.pageCount)
                    continue
                }
                let data = Data(bytes: ptr, count: pageLen)
                sc_free_bytes(ptr, pageLen)

                guard let image = NSImage(data: data),
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
