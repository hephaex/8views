#include <CoreFoundation/CoreFoundation.h>
#include <CoreServices/CoreServices.h>
#include <QuickLook/QuickLook.h>
#import <Cocoa/Cocoa.h>
#import "DTQuickComicCommon.h"
#include "main.h"
#import <WebPMac/TSSTWebPImageRep.h>
#import "sc_extras.h"

/* -----------------------------------------------------------------------------
   Generate a preview for file

   This function's job is to create preview for designated file
   ----------------------------------------------------------------------------- */

OSStatus GeneratePreviewForURL(void *thisInterface, QLPreviewRequestRef preview, CFURLRef url, CFStringRef contentTypeUTI, CFDictionaryRef options)
{
	@autoreleasepool {
		// TODO: implement kQLReturnHasMore somehow
		if (![NSImageRep imageRepClassForType:@"org.webmproject.webp"]) {
			[NSImageRep registerImageRepClass:[TSSTWebPImageRep class]];
		}

		// Use Rust to enumerate pages (natural-sorted, no XADMaster needed).
		NSString *archivePath = [(__bridge NSURL *)url path];
		int32_t qlErr = 0;
		ScPageList *pages = sc_archive_open_pages(archivePath.UTF8String, &qlErr);

		if (QLPreviewRequestIsCancelled(preview)) {
			sc_archive_pages_free(pages);
			return kQLReturnNoError;
		}

		if (pages && sc_page_list_count(pages) > 0)
		{
			uint32_t count = sc_page_list_count(pages);
			CGImageSourceRef pageSourceRef;
			CGImageRef currentImage;
			CGRect canvasRect;
			CGContextRef cgContext = QLPreviewRequestCreatePDFContext(preview, NULL, NULL, NULL);
			if(cgContext)
			{
				uint32_t counter = 0;
				NSDate *pageRenderStartTime = [NSDate date];
				NSDate *currentTime = nil;
				do
				{
					size_t pageLen = 0;
					int32_t pageErr = 0;
					uint8_t *pageBytes = sc_archive_read_page(archivePath.UTF8String, counter, &pageLen, &pageErr);
					if (!pageBytes) {
						counter++;
						continue;
					}
					NSData *fileData = [NSData dataWithBytes:pageBytes length:pageLen];
					sc_free_bytes(pageBytes, pageLen);

					pageSourceRef = CGImageSourceCreateWithData((CFDataRef)fileData, NULL);
					if (!pageSourceRef) {
						NSImage *img = [[NSImage alloc] initWithData:fileData];
						NSData *imgData = img.TIFFRepresentation;
						pageSourceRef = CGImageSourceCreateWithData((CFDataRef)imgData, NULL);
					}
					if (pageSourceRef) {
						currentImage = CGImageSourceCreateImageAtIndex(pageSourceRef, 0, NULL);
						if (currentImage) {
							canvasRect = CGRectMake(0, 0, CGImageGetWidth(currentImage), CGImageGetHeight(currentImage));
							CGContextBeginPage(cgContext, &canvasRect);
							CGContextDrawImage(cgContext, canvasRect, currentImage);
							CGContextEndPage(cgContext);
							CFRelease(currentImage);
						}
						CFRelease(pageSourceRef);
					}
					currentTime = [NSDate date];
					counter++;
					if (QLPreviewRequestIsCancelled(preview)) {
						CFRelease(cgContext);
						sc_archive_pages_free(pages);
						return kQLReturnNoError;
					}
				} while(1 > [currentTime timeIntervalSinceDate: pageRenderStartTime] && counter < count);

				QLPreviewRequestFlushContext(preview, cgContext);
				CFRelease(cgContext);
			}
		}
		sc_archive_pages_free(pages);
		return noErr;
	}
}

void CancelPreviewGeneration(void *thisInterface, QLPreviewRequestRef preview)
{
    // Implement only if supported
}
