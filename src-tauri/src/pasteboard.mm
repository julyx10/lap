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

void lap_free_string(const char* ptr) {
    if (ptr) free((void*)ptr);
}

}
