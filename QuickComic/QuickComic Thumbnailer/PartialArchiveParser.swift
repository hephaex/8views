//
//  PartialArchiveParser.swift
//  QuickComic Thumbnailer
//
//  Created by C.W. Betts on 12/15/22.
//  Copyright © 2022 Dancing Tortoise Software. All rights reserved.
//
//  Replaced by Rust sc_archive_open_pages + sc_archive_read_page in Sprint 22.
//  File retained to avoid Xcode project modification; class body is empty.
//

import Foundation

// PartialArchiveParser is no longer used — cover-name lookup now uses
// sc_archive_open_pages (Rust) in ThumbnailProvider.swift.
internal final class PartialArchiveParser {}
