//
//  TSSTManagedGroup.m
//  SimpleComic
//
//  Created by Alexander Rauchfuss on 6/2/07.
//  Copyright 2007 Dancing Tortoise Software. All rights reserved.
//

#import "TSSTManagedGroup.h"
#import "TSSTManagedGroup+CoreDataProperties.h"
#import "SimpleComicAppDelegate.h"
#import <XADMaster/XADArchive.h>
#import "sc_extras.h"
#import <Quartz/Quartz.h>
#import "TSSTImageUtilities.h"
#import "TSSTPage.h"
#import "TSSTPage+CoreDataProperties.h"

@interface TSSTManagedArchive () <XADArchiveDelegate>
-(void)archiveNeedsPassword:(XADArchive *)archive;

@end

@implementation TSSTManagedGroup

- (void)awakeFromInsert
{
	[super awakeFromInsert];
	groupLock = [NSLock new];
	instance = nil;
}

- (void)awakeFromFetch
{
	[super awakeFromFetch];
	groupLock = [NSLock new];
	instance = nil;
}

- (void)willTurnIntoFault
{
	NSError * error = nil;
	if(self.nested)
	{
		NSURL *fileURL = self.fileURL;
		if(fileURL != nil && ![[NSFileManager defaultManager] removeItemAtURL:fileURL  error: &error])
		{
			NSLog(@"%@",[error localizedDescription]);
		}
	}
	[self.fileURL stopAccessingSecurityScopedResource];
}

- (void)didTurnIntoFault
{
	instance = nil;
	groupLock = nil;
}

@synthesize fileURL=_url;

- (void)setFileURL:(NSURL *)fileURL
{
	if (_url && _url != fileURL) {
		[fileURL stopAccessingSecurityScopedResource];
	}
	_url = fileURL;
	NSError * urlError = nil;
	[fileURL startAccessingSecurityScopedResource];
	NSData * bookmarkData = [fileURL bookmarkDataWithOptions: NSURLBookmarkCreationWithSecurityScope | NSURLBookmarkCreationSecurityScopeAllowOnlyReadAccess
							  includingResourceValuesForKeys: @[NSURLVolumeURLForRemountingKey, NSURLVolumeUUIDStringKey]
											   relativeToURL: nil
													   error: &urlError];
	if (bookmarkData == nil || urlError != nil)
	{
		bookmarkData = nil;
		[NSApp presentError: urlError];
	}
	self.pathData = bookmarkData;
}

/// helper: common code of \c probeFileURL and \c fileURL
- (NSURL *)ResolvingURLWithStale:(BOOL *)stalep error:(NSError **)errorp {
	return self.pathData ? [NSURL URLByResolvingBookmarkData: self.pathData
												options: NSURLBookmarkResolutionWithoutUI | NSURLBookmarkResolutionWithSecurityScope
										  relativeToURL: nil
									bookmarkDataIsStale: stalep
												  error: errorp] : nil;
}

- (NSURL *)probeFileURL {
	if (_url && [_url checkResourceIsReachableAndReturnError:NULL]) {
		return _url;
	}
	NSError * urlError = nil;
	BOOL stale = NO;
	NSURL *fileURL = [self ResolvingURLWithStale:&stale error:&urlError];
	if (stale && fileURL) {
		// regenerate stale bookmark.
		self.fileURL = fileURL;
	} else if (fileURL) {
		//cache fileURL
		_url = fileURL;
	}
	return fileURL;;
}

- (NSURL *)fileURL
{
	if (_url && [_url checkResourceIsReachableAndReturnError:NULL]) {
		return _url;
	}
	NSError * urlError = nil;
	BOOL stale = NO;
	NSURL *fileURL = [self ResolvingURLWithStale:&stale error:&urlError];
	//For backwards compatibility
	if (fileURL == nil || urlError != nil) {
		NSError *othErr = nil;
		fileURL = [NSURL URLByResolvingBookmarkData: self.pathData
											options: 0
									  relativeToURL: nil
								bookmarkDataIsStale: &stale
											  error: &othErr];
		
		if (fileURL && othErr == nil) {
			NSOpenPanel *panel = [NSOpenPanel openPanel];
			panel.canChooseDirectories = YES;
			panel.allowsMultipleSelection = NO;
			//panel.expanded = YES;
			panel.message = [NSString stringWithFormat:NSLocalizedString(@"Please re-select '%@'", @"re-select file request"), fileURL.lastPathComponent];
			panel.directoryURL = [fileURL URLByDeletingLastPathComponent];
			
			if ([panel runModal] == NSModalResponseOK) {
				othErr = nil;
				NSData *bookmarkData = [panel.URL bookmarkDataWithOptions: NSURLBookmarkCreationWithSecurityScope | NSURLBookmarkCreationSecurityScopeAllowOnlyReadAccess
										   includingResourceValuesForKeys: @[NSURLVolumeURLForRemountingKey, NSURLVolumeUUIDStringKey]
															relativeToURL: nil
																	error: &othErr];
				
				if (bookmarkData) {
					self.pathData = bookmarkData;
					fileURL = [NSURL URLByResolvingBookmarkData: bookmarkData
														options: NSURLBookmarkResolutionWithoutUI | NSURLBookmarkResolutionWithSecurityScope
												  relativeToURL: nil
											bookmarkDataIsStale: &stale
														  error: &othErr];
					
				}
			} else {
				fileURL = nil;
			}
			urlError = othErr;
		}
	}
	
	if (fileURL == nil || urlError != nil)
	{
		fileURL = nil;
		[[self managedObjectContext] deleteObject: self];
		if (urlError) {
			[NSApp presentError: urlError];
		}
	}
	else if (stale)
	{
		// regenerate stale bookmark.
		self.fileURL = fileURL;
	} else {
		//cache fileURL
		_url = fileURL;
	}
	
	return fileURL;
}

- (void)setPath:(NSString *)newPath
{
	self.fileURL = [[NSURL alloc] initFileURLWithPath: newPath];
}


- (NSString *)path
{
	return self.fileURL.path;
}


- (id)instance
{
	return nil;
}

- (NSData *)dataForPageIndex:(NSInteger)index
{
	return nil;
}

- (void)requestDataForPageIndex:(NSInteger)index completionHandler:(void(^)(NSData *_Nullable pageData, NSError *_Nullable error))callback
{
	callback(nil, [NSError errorWithDomain:NSOSStatusErrorDomain code:unimpErr userInfo:nil]);
}

- (NSManagedObject *)topLevelGroup
{
	return self;
}

- (void)nestedFolderContents
{
	NSURL * folderPath = self.fileURL;
	NSFileManager * fileManager = [NSFileManager defaultManager];
	TSSTManagedGroup * nestedDescription;
	NSError * error = nil;
	NSArray<NSURL*> * nestedFiles = [fileManager contentsOfDirectoryAtURL:folderPath includingPropertiesForKeys:nil options:(NSDirectoryEnumerationSkipsSubdirectoryDescendants | NSDirectoryEnumerationSkipsHiddenFiles) error:&error];
	if (error)
	{
		NSLog(@"%@",[error localizedDescription]);
	}
	BOOL isDirectory;
	
	for (NSURL *path in nestedFiles)
	{
		nestedDescription = nil;
		NSString *fileExtension = [[path pathExtension] lowercaseString];
		BOOL exists = [fileManager fileExistsAtPath: path.path isDirectory: &isDirectory];
		if(exists && ![[[path lastPathComponent] substringToIndex: 1] isEqualToString: @"."])
		{
			if(isDirectory)
			{
				nestedDescription = [NSEntityDescription insertNewObjectForEntityForName: @"ImageGroup" inManagedObjectContext: [self managedObjectContext]];
				nestedDescription.fileURL = path;
				nestedDescription.name = path.relativePath ?: path.path;
				[nestedDescription nestedFolderContents];
			}
			else if([[TSSTManagedArchive archiveExtensions] containsObject: fileExtension])
			{
				nestedDescription = [NSEntityDescription insertNewObjectForEntityForName: @"Archive" inManagedObjectContext: [self managedObjectContext]];
				nestedDescription.fileURL = path;
				nestedDescription.name = path.relativePath ?: path.path;
				[(TSSTManagedArchive *)nestedDescription nestedArchiveContents];
			}
			else if([fileExtension isEqualToString: @"pdf"])
			{
				nestedDescription = [NSEntityDescription insertNewObjectForEntityForName: @"PDF" inManagedObjectContext: [self managedObjectContext]];
				nestedDescription.fileURL = path;
				nestedDescription.name = path.relativePath ?: path.path;
				[(TSSTManagedPDF *)nestedDescription pdfContents];
			}
			else if([[TSSTPage imageExtensions] containsObject: fileExtension])
			{
				nestedDescription = [NSEntityDescription insertNewObjectForEntityForName: @"Image" inManagedObjectContext: [self managedObjectContext]];
				[nestedDescription setValue: path.path forKey: @"imagePath"];
			}
			else if ([[TSSTPage textExtensions] containsObject: fileExtension])
			{
				nestedDescription = [NSEntityDescription insertNewObjectForEntityForName: @"Image" inManagedObjectContext: [self managedObjectContext]];
				[nestedDescription setValue: path.path forKey: @"imagePath"];
				[nestedDescription setValue: @YES forKey: @"text"];
			}
			
			if(nestedDescription)
			{
				[nestedDescription setValue: self forKey: @"group"];
			}
		}
	}
}

- (NSSet *)nestedImages
{
	NSMutableSet * allImages = [self.images mutableCopy];
	NSSet * groups = self.groups;
	for(TSSTManagedGroup * group in groups)
	{
		[allImages unionSet: group.nestedImages];
	}
	
	return allImages;
}

@end


@implementation TSSTManagedArchive

+ (NSArray *)archiveExtensions
{
	static NSArray * extensions = nil;
	static dispatch_once_t onceToken;
	dispatch_once(&onceToken, ^{
		NSArray *archives = self.archiveTypes;
		NSMutableSet<NSString*> *aimageTypes = [[NSMutableSet alloc] initWithCapacity:archives.count];
		for (NSString *uti in archives) {
			NSArray *fileExts =
			CFBridgingRelease(UTTypeCopyAllTagsWithClass((__bridge CFStringRef)uti, kUTTagClassFilenameExtension));
			[aimageTypes addObjectsFromArray:fileExts];
		}
		extensions = [[aimageTypes allObjects] sortedArrayUsingSelector:@selector(compare:)];
	});
	
	return extensions;
}

+ (NSArray*)archiveTypes
{
	static NSArray * extensions = nil;
	static dispatch_once_t onceToken;
	dispatch_once(&onceToken, ^{
		// TODO: have this expansive?
		extensions = @[@"com.rarlab.rar-archive", @"cx.c3.cbr-archive",
					   (NSString*)kUTTypeZipArchive, @"cx.c3.cbz-archive",
					   @"org.7-zip.7-zip-archive", @"cx.c3.cb7-archive",
					   @"public.archive.lha", @"cx.c3.lha-archive",
					   @"com.dancingtortoise.simplecomic.cbt", @"public.tar-archive",
					   @"com.yacreader.yacreader.cbr", @"com.yacreader.yacreader.cbz",
					   @"com.simplecomic.cbz-archive", @"com.simplecomic.cbr-archive",
					   @"com.simplecomic.cb7-archive", @"com.simplecomic.cbt-archive"];
	});
	
	return extensions;
}

+ (NSArray *)quicklookExtensions
{
	static NSArray * extensions = nil;
	static dispatch_once_t onceToken;
	dispatch_once(&onceToken, ^{
		extensions = @[@"cbr", @"cbz", @"cbt", @"cb7"];
	});
	
	return extensions;
}

- (void)willTurnIntoFault
{
	NSError * error;
	if(self.nested)
	{
		if(![[NSFileManager defaultManager] removeItemAtPath: self.path error: &error])
		{
			NSLog(@"%@",[error localizedDescription]);
		}
	}
	
	NSString * solid  = self.solidDirectory;
	if(solid)
	{
		if(![[NSFileManager defaultManager] removeItemAtPath: solid error: &error])
		{
			NSLog(@"%@",[error localizedDescription]);
		}
	}
}

- (id)instance
{
	if (!instance)
	{
		NSURL *aFileURL = self.fileURL;
		if([aFileURL checkResourceIsReachableAndReturnError:NULL])
		{
			[aFileURL startAccessingSecurityScopedResource];
			instance = [[XADArchive alloc] initWithFileURL: aFileURL delegate: self error:NULL];
			
			// Set the archive delegate so that password and encoding queries can have a modal pop up.
			
			if(self.password)
			{
				[instance setPassword: self.password];
			}
		}
	}
	
	return instance;
}


// TODO(phase6/sprint13): solid RAR archives require sequential decompression.
// Rust opens a fresh archive handle per call, so solid RAR page N costs O(N) decompressions.
// The old XADMaster solidDirectory disk-cache handled this; a Rust-side page-cache or
// bulk-prefetch API is needed before removing XADMaster entirely for RAR archives.
- (void)requestDataForPageIndex:(NSInteger)index completionHandler:(void(^)(NSData *_Nullable pageData, NSError *_Nullable error))callback
{
	NSString *archivePath = self.fileURL.path;
	if (!archivePath) {
		callback(nil, [NSError errorWithDomain:NSCocoaErrorDomain code:NSFileNoSuchFileError userInfo:nil]);
		return;
	}
	size_t outLen = 0;
	int32_t errCode = 0;
	uint8_t *buf = sc_archive_read_page(archivePath.UTF8String, (uint32_t)index, &outLen, &errCode);
	if (!buf) {
		NSError *err = [NSError errorWithDomain:SCArchiveErrorDomain code:errCode userInfo:nil];
		callback(nil, err);
		return;
	}
	NSData *imageData = [NSData dataWithBytes:buf length:outLen];
	sc_free_bytes(buf, outLen);
	callback(imageData, nil);
}


- (NSManagedObject *)topLevelGroup
{
	NSManagedObject * group = self;
	NSManagedObject * parentGroup = group;
	
	while(group)
	{
		group = [group valueForKeyPath: @"group"];
		parentGroup = group && [group isMemberOfClass:[TSSTManagedArchive class]] ? group : parentGroup;
	}
	
	return parentGroup;
}

- (void)nestedArchiveContents
{
	NSString *archivePath = self.fileURL.path;
	if (!archivePath) return;

	// Phase 1: Enumerate image pages via Rust.
	// sc_archive_open_pages returns only image entries in natural-sort order.
	// The loop index i is the Rust-side index used by sc_archive_read_page — must match.
	int32_t errCode = 0;
	ScPageList *pages = sc_archive_open_pages(archivePath.UTF8String, &errCode);
	if (pages) {
		uint32_t count = sc_page_list_count(pages);
		for (uint32_t i = 0; i < count; i++) {
			const char *rawName = sc_page_list_name(pages, i);
			if (!rawName) continue;
			NSString *fileName = [NSString stringWithUTF8String:rawName];
			if (!fileName || fileName.length == 0) continue;
			if ([[[fileName lastPathComponent] substringToIndex:1] isEqualToString:@"."]) continue;

			TSSTManagedGroup *entry = [NSEntityDescription
				insertNewObjectForEntityForName:@"Image"
				inManagedObjectContext:[self managedObjectContext]];
			[entry setValue:fileName forKey:@"imagePath"];
			[entry setValue:@(i) forKey:@"index"];
			entry.group = self;
		}
		sc_archive_pages_free(pages);
	}

	// Phase 2: Handle nested archives and PDFs via XADMaster (edge cases only).
	// TODO(phase6/sprint14): replace with Rust bulk-read API when sc_archive_read_page
	// supports non-image entry extraction.
	XADArchive *imageArchive = self.instance;
	if (!imageArchive) return;

	NSFileManager *fileManager = [NSFileManager defaultManager];
	NSData *fileData;
	NSInteger collision = 0;
	NSString *tempPath = nil;

	for (NSInteger counter = 0; counter < [imageArchive numberOfEntries]; ++counter) {
		NSString *fileName = [imageArchive nameOfEntry:counter];
		if (!fileName || fileName.length == 0) continue;
		if ([[[fileName lastPathComponent] substringToIndex:1] isEqualToString:@"."]) continue;

		NSString *extension = [[fileName pathExtension] lowercaseString];
		TSSTManagedGroup *nestedDescription = nil;

		if ([[TSSTPage imageExtensions] containsObject:extension]) {
			// Already handled by Rust above — skip
			continue;
		} else if ([[TSSTManagedArchive archiveExtensions] containsObject:extension]) {
			fileData = [imageArchive contentsOfEntry:counter];
			nestedDescription = [NSEntityDescription insertNewObjectForEntityForName:@"Archive"
				inManagedObjectContext:[self managedObjectContext]];
			nestedDescription.name = fileName;
			nestedDescription.nested = YES;

			collision = 0;
			do {
				tempPath = [NSString stringWithFormat:@"%li-%@", (long)collision, fileName];
				tempPath = [NSTemporaryDirectory() stringByAppendingPathComponent:tempPath];
				++collision;
			} while ([fileManager fileExistsAtPath:tempPath]);

			[[NSFileManager defaultManager] createDirectoryAtPath:[tempPath stringByDeletingLastPathComponent]
									  withIntermediateDirectories:YES attributes:nil error:NULL];
			[[NSFileManager defaultManager] createFileAtPath:tempPath contents:fileData attributes:nil];
			nestedDescription.path = tempPath;
			[(TSSTManagedArchive *)nestedDescription nestedArchiveContents];
		} else if ([extension isEqualToString:@"pdf"]) {
			NSString *fullFileName = fileName;
			fileName = [fileName lastPathComponent];
			nestedDescription = [NSEntityDescription insertNewObjectForEntityForName:@"PDF"
				inManagedObjectContext:[self managedObjectContext]];
			tempPath = [NSTemporaryDirectory() stringByAppendingPathComponent:fileName];
			collision = 0;
			while ([fileManager fileExistsAtPath:tempPath]) {
				++collision;
				fileName = [NSString stringWithFormat:@"%li-%@", (long)collision, fileName];
				tempPath = [NSTemporaryDirectory() stringByAppendingPathComponent:fileName];
			}
			fileData = [imageArchive contentsOfEntry:counter];
			[fileData writeToFile:tempPath atomically:YES];
			nestedDescription.path = tempPath;
			nestedDescription.nested = YES;
			nestedDescription.name = fullFileName;
			[(TSSTManagedPDF *)nestedDescription pdfContents];
		}

		if (nestedDescription) {
			nestedDescription.group = self;
		}
	}
}


- (BOOL)quicklookCompatible
{	
	NSString * extension = [[self.name pathExtension] lowercaseString];
	return [TSSTManagedArchive.quicklookExtensions containsObject: extension];
}


/* Delegates */

/**  Called when Simple Comic encounters a password protected
 archive.  Brings a password dialog forward. */
- (void)archiveNeedsPassword:(XADArchive *)archive
{
	NSString * password = self.password;
	
	if(password)
	{
		archive.password = password;
		return;
	}
	
	password = [(SimpleComicAppDelegate*)[NSApp delegate] passwordForArchiveWithPath: self.path];
	archive.password = password;
	
	self.password = password;
}

@end


@implementation TSSTManagedPDF

- (id)instance
{
	if (!instance)
	{
		NSURL *aURL = self.fileURL;
		[aURL startAccessingSecurityScopedResource];
		instance = [[PDFDocument alloc] initWithURL: aURL];
	}
	
	return instance;
}

- (void)requestDataForPageIndex:(NSInteger)index completionHandler:(void(^)(NSData *_Nullable pageData, NSError *_Nullable error))callback
{
	[groupLock lock];
	PDFPage * page = [(PDFDocument*)[self instance] pageAtIndex: index];
	[groupLock unlock];
	
	NSRect bounds = [page boundsForBox: kPDFDisplayBoxMediaBox];
	CGFloat dimension = 1400;
	CGFloat scale = 1 > (NSHeight(bounds) / NSWidth(bounds)) ? dimension / NSWidth(bounds) :  dimension / NSHeight(bounds);
	bounds.size = scaleSize(bounds.size, scale);
	if (NSEqualRects(bounds, NSZeroRect)) {
		// Prevent zero size exception for images
		bounds.size = NSMakeSize(50, 50);
	}
	if (isinf(scale) || scale == 0) {
		scale = 1;
	}
	
	NSImage * pageImage = [[NSImage alloc] initWithSize: bounds.size];
	[pageImage lockFocus];
		[[NSColor whiteColor] set];
		NSRectFill(bounds);
		NSAffineTransform * scaleTransform = [NSAffineTransform transform];
		[scaleTransform scaleBy: scale];
		[scaleTransform concat];
		[page drawWithBox: kPDFDisplayBoxMediaBox toContext:[[NSGraphicsContext currentContext] CGContext]];
	[pageImage unlockFocus];
	
	NSData * imageData = [pageImage TIFFRepresentation];
	
	callback(imageData, nil);
}

- (void)pdfContents
{
	PDFDocument * rep = [self instance];
	TSSTPage * imageDescription;
	NSMutableSet<TSSTPage*> * pageSet = [NSMutableSet set];
	NSInteger imageCount = [rep pageCount];
	for (NSInteger pageNumber = 0; pageNumber < imageCount; ++pageNumber)
	{
		imageDescription = [NSEntityDescription insertNewObjectForEntityForName: @"Image" inManagedObjectContext: [self managedObjectContext]];
		imageDescription.imagePath = [NSString stringWithFormat: NSLocalizedString(@"PDF page %li", @"PDF page number"), (long)(pageNumber + 1)];
		imageDescription.index = @(pageNumber);
		[pageSet addObject: imageDescription];
	}
	self.images = pageSet;
}

@end
