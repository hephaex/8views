/*
Copyright (c) 2006-2009 Dancing Tortoise Software

	Permission is hereby granted, free of charge, to any person
	obtaining a copy of this software and associated documentation
	files (the "Software"), to deal in the Software without
	restriction, including without limitation the rights to use,
	copy, modify, merge, publish, distribute, sublicense, and/or
	sell copies of the Software, and to permit persons to whom the
	Software is furnished to do so, subject to the following
	conditions:

	The above copyright notice and this permission notice shall be
	included in all copies or substantial portions of the Software.

  TSSTPage.m
*/

#import "TSSTPage.h"
#import "SimpleComicAppDelegate.h" // AppleScript
#import "TSSTSessionWindowController.h"	// AppleScript
#import "TSSTImageUtilities.h"
#import "TSSTManagedGroup.h"
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#import "sc_extras.h"

static NSDictionary * TSSTInfoPageAttributes = nil;
static NSSize monospaceCharacterSize;

@implementation TSSTPage

+ (NSArray<NSString*>*)imageTypes
{
	static NSArray * imageTypes = nil;
	static dispatch_once_t onceToken;
	dispatch_once(&onceToken, ^{
		NSMutableArray<NSString*> *aimageTypes = [[NSImage imageTypes] mutableCopy];
		[aimageTypes removeObject:(NSString*)kUTTypePDF];
		[aimageTypes filterUsingPredicate:[NSPredicate predicateWithFormat:@"!(SELF like %@)" argumentArray:@[@"com.adobe.encapsulated-postscript"]]];
		imageTypes = [aimageTypes copy];
	});
	
	return imageTypes;
}

+ (NSArray *)imageExtensions
{
	static NSArray * imageTypes = nil;
	static dispatch_once_t onceToken;
	dispatch_once(&onceToken, ^{
		NSArray<NSString*> *imgTyp = self.imageTypes;
		NSMutableSet<NSString*> *aimageTypes = [[NSMutableSet alloc] initWithCapacity:imgTyp.count * 2];
		for (NSString *uti in imgTyp) {
			UTType *fileUTI = [UTType typeWithIdentifier:uti];
			NSArray *fileExts = fileUTI.tags[UTTagClassFilenameExtension];
			[aimageTypes addObjectsFromArray:fileExts];
		}
		//Some early JPEGs have the extension jfi/jfif.
		[aimageTypes addObject:@"jfi"];
		[aimageTypes addObject:@"jfif"];
		imageTypes = [[aimageTypes allObjects] sortedArrayUsingSelector:@selector(compare:)];
	});
	
	return imageTypes;
}

+ (NSArray *)textExtensions
{
	static NSArray * textTypes = nil;
	static dispatch_once_t onceToken;
	dispatch_once(&onceToken, ^{
		textTypes = @[@"txt", @"nfo", @"info"];
	});
	
	return textTypes;
}

+ (void)initialize
{
	static dispatch_once_t onceToken;
	dispatch_once(&onceToken, ^{
		/* Figure out the size of a single monospace character to set the tab stops */
		NSFont * menlo14 = [NSFont fontWithName: @"Menlo" size: 14];
		NSDictionary * fontAttributes = @{NSFontAttributeName: menlo14};
		monospaceCharacterSize = [@"A" boundingRectWithSize: NSZeroSize options: 0 attributes: fontAttributes].size;
		
		NSMutableArray<NSTextTab*> * tabStops = [NSMutableArray arrayWithCapacity:120/8-1];
		/* Loop through the tab stops */
		for (NSInteger tabSize = 8; tabSize < 120; tabSize+=8) {
			CGFloat tabLocation = tabSize * monospaceCharacterSize.width;
			NSTextTab * tabStop = [[NSTextTab alloc] initWithType: NSLeftTabStopType location: tabLocation];
			[tabStops addObject: tabStop];
		}
		
		NSMutableParagraphStyle * style = [[NSParagraphStyle defaultParagraphStyle] mutableCopy];
		style.tabStops = tabStops;
		
		TSSTInfoPageAttributes = @{NSFontAttributeName: menlo14,
								   NSParagraphStyleAttributeName: style};
	});
}

- (void)awakeFromInsert
{
	[super awakeFromInsert];
	thumbLock = [NSLock new];
	loaderLock = [NSLock new];
}

- (void)awakeFromFetch
{
	[super awakeFromFetch];
	thumbLock = [NSLock new];
	loaderLock = [NSLock new];
}

- (void)didTurnIntoFault
{
	loaderLock = nil;
	thumbLock = nil;
}

- (BOOL)shouldDisplayAlone
{   
	if(self.text)
	{
		return YES;
	}
	
	CGFloat defaultAspect = 1;
	CGFloat aspect = self.aspectRatio;
	if(!aspect)
	{
		NSData * imageData = [self pageData];
		[self setOwnSizeInfoWithData: imageData];
		aspect = self.aspectRatio;
	}
	
	return aspect != 0 ? aspect > defaultAspect : YES;
}

- (void)setOwnSizeInfoWithData:(NSData *)imageData
{
	CGFloat aspect;
	NSSize imageSize;
	// Try NSBitmapImageRep first.
	NSImageRep * pageRep = [NSBitmapImageRep imageRepWithData: imageData];
	if (!pageRep) {
		// If it failed, try iterating through each registered NSImageRep subclass.
		Class imgRepClass = [NSImageRep imageRepClassForData:imageData];
		pageRep = [imgRepClass imageRepWithData: imageData];
	}
	imageSize = NSMakeSize([pageRep pixelsWide], [pageRep pixelsHigh]);
	
	if(!NSEqualSizes(NSZeroSize, imageSize))
	{
		aspect = imageSize.width / imageSize.height;
		self.width = imageSize.width;
		self.height = imageSize.height;
		self.aspectRatio = aspect;
	}
}

- (NSString *)name
{
	return [self.imagePath lastPathComponent];
}

- (NSImage *)thumbnail
{
	NSImage * thumbnail = nil;
	NSData * thumbnailData = self.thumbnailData;
	if(!thumbnailData)
	{
		thumbnailData = [self prepThumbnail];
		self.thumbnailData = thumbnailData;
		thumbnail = [[NSImage alloc] initWithData: thumbnailData];
	}
	else
	{
		thumbnail = [[NSImage alloc] initWithData: thumbnailData];
	}
	
	return thumbnail;
}

- (NSData *)prepThumbnail
{
	[thumbLock lock];

	NSData *thumbnailData = nil;
	NSData *rawPageData = [self pageData];

	if (rawPageData) {
		// Use Rust Lanczos3 thumbnail generation (faster, no AppKit overhead).
		size_t outLen = 0;
		uint32_t outW = 0, outH = 0;
		int32_t errCode = 0;
		uint8_t *rgba = sc_thumbnail_from_bytes(
			(const uint8_t *)rawPageData.bytes, rawPageData.length,
			256, &outLen, &outW, &outH, &errCode);

		if (rgba && outW > 0 && outH > 0) {
			// Rust to_rgba8().into_raw() → R,G,B,A order (alpha LAST, non-premultiplied).
			// NSBitmapFormatAlphaNonpremultiplied = alpha last + non-premultiplied.
			// NSBitmapFormatAlphaFirst would be ARGB (alpha FIRST) — wrong for Rust RGBA.
			// NSCalibratedRGBColorSpace = sRGB with color management (correct for JPEG/PNG
			// source images). NSDeviceRGBColorSpace bypasses color management.
			NSBitmapImageRep *rep = [[NSBitmapImageRep alloc]
				initWithBitmapDataPlanes: NULL
							  pixelsWide: outW
							  pixelsHigh: outH
						   bitsPerSample: 8
						 samplesPerPixel: 4
								hasAlpha: YES
								isPlanar: NO
						  colorSpaceName: NSCalibratedRGBColorSpace
							bitmapFormat: NSBitmapFormatAlphaNonpremultiplied
							 bytesPerRow: outW * 4
							bitsPerPixel: 32];
			if (rep) {
				memcpy([rep bitmapData], rgba, outLen);
				thumbnailData = [rep TIFFRepresentation];
			}
			sc_free_bytes(rgba, outLen);
		}
	}

	// Fallback: AppKit scaling if Rust fails (e.g. animated GIF).
	if (!thumbnailData) {
		NSImage *managedImage = [self pageImage];
		if (managedImage) {
			NSSize pixelSize = sizeConstrainedByDimension([managedImage size], 256);
			NSImage *temp = [[NSImage alloc] initWithSize: pixelSize];
			[temp lockFocus];
			[[NSGraphicsContext currentContext] setImageInterpolation: NSImageInterpolationHigh];
			[managedImage drawInRect: NSMakeRect(0, 0, pixelSize.width, pixelSize.height)
							fromRect: NSZeroRect
						   operation: NSCompositingOperationSourceOver
							fraction: 1.0];
			[temp unlockFocus];
			thumbnailData = [temp TIFFRepresentation];
		}
	}

	[thumbLock unlock];
	return thumbnailData;
}

/// Maximum pixel dimension for in-memory storage. Images larger than this are
/// pre-scaled with Rust Lanczos3 to reduce memory pressure before passing to
/// CoreAnimation. CALayer handles the final GPU display scaling.
static const uint32_t kSCMaxInMemoryDimension = 2048;

- (NSImage *)pageImage
{
	if(self.text)
	{
		return [self textPage];
	}

	NSData *imageData = [self pageData];
	if (!imageData) return nil;

	[self setOwnSizeInfoWithData: imageData];

	// Attempt Rust pre-scale for large images (>2048px) to reduce memory pressure.
	size_t outLen = 0; uint32_t outW = 0, outH = 0; int32_t errCode = 0;
	uint8_t *rgba = sc_cap_image_bytes(
		(const uint8_t *)imageData.bytes, imageData.length,
		kSCMaxInMemoryDimension, &outLen, &outW, &outH, &errCode);

	NSImage *imageFromData = nil;

	if (rgba) {
		// Large image — use Rust-scaled RGBA as the source.
		// NSCalibratedRGBColorSpace = sRGB; NSBitmapFormatAlphaNonpremultiplied = RGBA order.
		NSBitmapImageRep *rep = [[NSBitmapImageRep alloc]
			initWithBitmapDataPlanes: NULL
						  pixelsWide: outW
						  pixelsHigh: outH
					   bitsPerSample: 8
					 samplesPerPixel: 4
							hasAlpha: YES
							isPlanar: NO
					  colorSpaceName: NSCalibratedRGBColorSpace
						bitmapFormat: NSBitmapFormatAlphaNonpremultiplied
						 bytesPerRow: outW * 4
						bitsPerPixel: 32];
		if (rep) {
			memcpy([rep bitmapData], rgba, outLen);
			imageFromData = [[NSImage alloc] initWithSize:NSMakeSize(outW, outH)];
			[imageFromData addRepresentation:rep];
		}
		sc_free_bytes(rgba, outLen);
	}

	// Fallback: AppKit native decoding (animated GIF, small images, Rust decode failure).
	if (!imageFromData) {
		imageFromData = [[NSImage alloc] initWithData: imageData];
	}

	NSSize imageSize = NSMakeSize(self.width, self.height);
	if (!imageFromData || NSEqualSizes(NSZeroSize, imageSize)) {
		return nil;
	}

	// Apply physical pixel size so AppKit knows the true dimensions.
	if (rgba) {
		// Already scaled — update stored size to match capped dimensions.
		self.width  = outW;
		self.height = outH;
		imageSize = NSMakeSize(outW, outH);
	}
	[imageFromData setCacheMode: NSImageCacheNever];
	[imageFromData setSize: imageSize];
	[imageFromData setCacheMode: NSImageCacheBySize];

	return imageFromData;
}

- (NSImage *)textPage
{
	__block NSData * textData;
	if(self.index != nil)
	{
		[self.group requestDataForPageIndex: [self.index integerValue] completionHandler:^(NSData * _Nullable pageData, NSError * _Nullable error) {
			textData = pageData;
		}];
	}
	else
	{
		textData = [NSData dataWithContentsOfFile: self.imagePath];
	}
	
	BOOL lossyConversion = NO;
	NSString * text;
	NSStringEncoding stringEncoding = [NSString stringEncodingForData: textData
													  encodingOptions: @{NSStringEncodingDetectionFromWindowsKey: @YES}
													  convertedString: &text
												  usedLossyConversion: &lossyConversion];
	if (stringEncoding == 0 && text == nil) {
		// get back something, even if it's garbled.
		stringEncoding = NSMacOSRomanStringEncoding;
		text = [[NSString alloc] initWithData: textData encoding: stringEncoding];
	}
	//	int lineCount = 0;
	NSRect lineRect;
	NSRect pageRect = NSZeroRect;
	
	NSUInteger index = 0;
	NSUInteger textLength = [text length];
	NSRange lineRange;
	NSString * singleLine;
	while(index < textLength)
	{
		lineRange = [text lineRangeForRange: NSMakeRange(index, 0)];
		index = NSMaxRange(lineRange);
		singleLine = [text substringWithRange: lineRange];
		lineRect = [singleLine boundingRectWithSize: NSMakeSize(800, 800) options: NSStringDrawingUsesLineFragmentOrigin attributes: TSSTInfoPageAttributes];
		if(NSWidth(lineRect) > NSWidth(pageRect))
		{
			pageRect.size.width = lineRect.size.width;
		}
		
		pageRect.size.height += (NSHeight(lineRect) - 19);
	}
	pageRect.size.width += 10;
	pageRect.size.height += 10;
	pageRect.size.height = MAX(NSHeight(pageRect), 500);
	
	NSImage * textImage = [[NSImage alloc] initWithSize: pageRect.size];
	
	[textImage lockFocus];
	[[NSColor whiteColor] set];
	NSRectFill(pageRect);
	[text drawWithRect: NSInsetRect( pageRect, 5, 5) options: NSStringDrawingUsesLineFragmentOrigin attributes: TSSTInfoPageAttributes];
	[textImage unlockFocus];
	
	return textImage;
}

- (NSData *)pageData
{
	__block NSData * imageData = nil;
	TSSTManagedGroup * group = self.group;
	if(self.index != nil)
	{
		NSInteger entryIndex = [self.index integerValue];
		[group requestDataForPageIndex:entryIndex completionHandler:^(NSData * _Nullable pageData, NSError * _Nullable error) {
			imageData = pageData;
		}];
	}
	else if([self imagePath])
	{
		imageData = [NSData dataWithContentsOfFile: self.imagePath];
	}
	
	return imageData;
}

#pragma mark - Applescript

- (NSScriptObjectSpecifier *)objectSpecifier
{
	TSSTManagedSession *session = self.session;
	NSArray<TSSTSessionWindowController *> *controllers = [(SimpleComicAppDelegate *)[NSApp delegate] sessions];
	for (TSSTSessionWindowController *controller in controllers) {
		if (controller.session == session) {
			NSScriptObjectSpecifier *parent = [controller objectSpecifier];
			NSScriptClassDescription *desc = [NSScriptClassDescription classDescriptionForClass:[self class]];
			return [[NSIndexSpecifier alloc] initWithContainerClassDescription:desc
																											containerSpecifier:parent
																																		 key:@"page"
																																	 index:[self.index integerValue]];
		}
	}
	return nil;
}

@end
