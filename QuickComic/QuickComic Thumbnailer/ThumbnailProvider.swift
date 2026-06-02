//
//  ThumbnailProvider.swift
//  QuickComic Thumbnailer
//
//  Created by C.W. Betts on 12/15/22.
//  Copyright © 2022 Dancing Tortoise Software. All rights reserved.
//

import QuickLookThumbnailing

class ThumbnailProvider: QLThumbnailProvider {

    override func provideThumbnail(for request: QLFileThumbnailRequest, _ handler: @escaping (QLThumbnailReply?, Error?) -> Void) {
        let archiveURL = request.fileURL
        let archivePath = archiveURL.path

        do {
            let coverName = (try? UKXattrMetadataStore.string(forKey: SCQuickLookCoverName, atPath: archivePath, traverseLink: false)) ?? ""
            let coverRectString = (try? UKXattrMetadataStore.string(forKey: SCQuickLookCoverRect, atPath: archivePath, traverseLink: false)) ?? ""

            var imageData: Data? = nil
            var cropRect = CGRect.zero

            if !coverName.isEmpty {
                // xattr specifies a named cover image — find it in the archive
                if !coverRectString.isEmpty {
                    cropRect = NSRectFromString(coverRectString)
                }
                var errCode: Int32 = 0
                guard let pageList = sc_archive_open_pages(archivePath, &errCode) else {
                    throw CocoaError(.fileReadCorruptFile, userInfo: [NSURLErrorKey: archiveURL])
                }
                defer { sc_archive_pages_free(pageList) }

                let count = sc_page_list_count(pageList)
                for i in 0..<count {
                    guard let nameCStr = sc_page_list_name(pageList, i),
                          String(cString: nameCStr) == coverName else { continue }
                    var outLen = 0
                    var readErr: Int32 = 0
                    if let ptr = sc_archive_read_page(archivePath, i, &outLen, &readErr) {
                        imageData = Data(bytes: ptr, count: outLen)
                        sc_free_bytes(ptr, outLen)
                    }
                    break
                }
            } else {
                // No cover specified — use the optimised first-image path
                var outLen = 0
                var errCode: Int32 = 0
                if let ptr = sc_archive_read_first_image(archivePath, &outLen, &errCode) {
                    imageData = Data(bytes: ptr, count: outLen)
                    sc_free_bytes(ptr, outLen)
                }
            }

            guard let imageData, let image = NSImage(data: imageData) else {
                throw CocoaError(.fileReadCorruptFile, userInfo: [NSURLErrorKey: archiveURL])
            }

            var imageSize = cropRect.isEmpty ? image.size : cropRect.size
            imageSize = constrainSize(imageSize, dimension: request.maximumSize.height)

            let reply = QLThumbnailReply(contextSize: imageSize, currentContextDrawing: { () -> Bool in
                var canvasRect: CGRect = .zero
                var drawRect: CGRect = .zero

                if cropRect.isEmpty {
                    canvasRect = NSRect(origin: .zero, size: imageSize)
                    drawRect = NSRect(origin: .zero, size: fitSize(imageSize, in: image.size))
                } else {
                    canvasRect.size = fitSize(imageSize, in: cropRect.size)
                    let vertScale = canvasRect.size.height / image.size.height
                    let horScale = canvasRect.size.width / image.size.width
                    drawRect.origin = CGPoint(x: -(cropRect.origin.x), y: -(cropRect.origin.y))
                    drawRect.size = CGSize(width: cropRect.size.width / horScale, height: cropRect.size.height / vertScale)
                    canvasRect.size = imageSize
                }

                image.draw(in: canvasRect, from: drawRect, operation: .copy, fraction: 1)
                return true
            })

            handler(reply, nil)
        } catch {
            handler(nil, error)
        }
    }
}
