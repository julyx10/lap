#import <Cocoa/Cocoa.h>

extern "C" {

const char* lap_get_drag_image_url(void) {
    @autoreleasepool {
        NSPasteboard *pb = [NSPasteboard pasteboardWithName:NSPasteboardNameDrag];
        if (!pb) return NULL;

        NSArray<NSURL *> *urls = [pb readObjectsForClasses:@[[NSURL class]] options:nil];
        for (NSURL *url in urls) {
            NSString *str = [url absoluteString];
            if (str && ([str hasPrefix:@"http://"] || [str hasPrefix:@"https://"])) {
                return strdup([str UTF8String]);
            }
        }
        return NULL;
    }
}

const char* lap_get_drag_file_paths(void) {
    @autoreleasepool {
        NSPasteboard *pb = [NSPasteboard pasteboardWithName:NSPasteboardNameDrag];
        if (!pb) return NULL;

        NSArray<NSURL *> *urls = [pb readObjectsForClasses:@[[NSURL class]] options:nil];
        NSMutableArray<NSString *> *paths = [NSMutableArray array];
        for (NSURL *url in urls) {
            if (url.isFileURL && url.path) {
                [paths addObject:url.path];
            }
        }
        if (paths.count == 0) return NULL;

        NSData *json = [NSJSONSerialization dataWithJSONObject:paths options:0 error:nil];
        if (!json) return NULL;
        NSString *value = [[NSString alloc] initWithData:json encoding:NSUTF8StringEncoding];
        return value ? strdup([value UTF8String]) : NULL;
    }
}

bool lap_copy_files_and_image_to_clipboard(
    const char* file_paths_json,
    const unsigned char* image_bytes,
    size_t image_length
) {
    @autoreleasepool {
        if (!file_paths_json) return false;
        NSData *jsonData = [[NSString stringWithUTF8String:file_paths_json]
            dataUsingEncoding:NSUTF8StringEncoding];
        NSArray<NSString *> *paths = [NSJSONSerialization JSONObjectWithData:jsonData options:0 error:nil];
        if (![paths isKindOfClass:[NSArray class]] || paths.count == 0) return false;

        NSData *imageData = nil;
        NSImage *image = nil;
        if (image_bytes && image_length > 0) {
            imageData = [NSData dataWithBytes:image_bytes length:image_length];
            image = [[NSImage alloc] initWithData:imageData];
        }

        NSMutableArray *objects = [NSMutableArray arrayWithCapacity:paths.count];
        for (NSUInteger index = 0; index < paths.count; index++) {
            NSString *path = paths[index];
            if ([path isKindOfClass:[NSString class]]) {
                NSURL *fileURL = [NSURL fileURLWithPath:path];
                if (index == 0 && image) {
                    NSPasteboardItem *item = [[NSPasteboardItem alloc] init];
                    [item setString:[fileURL absoluteString] forType:NSPasteboardTypeFileURL];
                    [item setData:imageData forType:NSPasteboardTypePNG];
                    NSData *tiffData = [image TIFFRepresentation];
                    if (tiffData) {
                        [item setData:tiffData forType:NSPasteboardTypeTIFF];
                    }
                    [objects addObject:item];
                } else {
                    [objects addObject:fileURL];
                }
            }
        }
        if (objects.count == 0) return false;

        NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
        [pasteboard clearContents];
        return [pasteboard writeObjects:objects];
    }
}

void lap_free_string(const char* ptr) {
    if (ptr) free((void*)ptr);
}

}
