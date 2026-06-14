#include <gtk/gtk.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    guchar *uri_list;
    gint uri_list_len;
    guchar *png;
    gint png_len;
} LapClipboardData;

static void lap_clipboard_get(
    GtkClipboard *clipboard,
    GtkSelectionData *selection_data,
    guint info,
    gpointer user_data
) {
    (void)clipboard;
    LapClipboardData *data = (LapClipboardData *)user_data;
    if (info == 0) {
        gtk_selection_data_set(
            selection_data,
            gdk_atom_intern_static_string("text/uri-list"),
            8,
            data->uri_list,
            data->uri_list_len
        );
    } else if (data->png && data->png_len > 0) {
        gtk_selection_data_set(
            selection_data,
            gdk_atom_intern_static_string("image/png"),
            8,
            data->png,
            data->png_len
        );
    }
}

static void lap_clipboard_clear(GtkClipboard *clipboard, gpointer user_data) {
    (void)clipboard;
    LapClipboardData *data = (LapClipboardData *)user_data;
    free(data->uri_list);
    free(data->png);
    free(data);
}

bool lap_copy_files_and_image_to_clipboard(
    const unsigned char *uri_list,
    size_t uri_list_len,
    const unsigned char *png,
    size_t png_len
) {
    if (!uri_list || uri_list_len == 0) return false;

    LapClipboardData *data = calloc(1, sizeof(LapClipboardData));
    if (!data) return false;

    data->uri_list = malloc(uri_list_len);
    if (!data->uri_list) {
        free(data);
        return false;
    }
    memcpy(data->uri_list, uri_list, uri_list_len);
    data->uri_list_len = (gint)uri_list_len;

    if (png && png_len > 0) {
        data->png = malloc(png_len);
        if (!data->png) {
            lap_clipboard_clear(NULL, data);
            return false;
        }
        memcpy(data->png, png, png_len);
        data->png_len = (gint)png_len;
    }

    GtkTargetEntry targets[] = {
        { "text/uri-list", 0, 0 },
        { "image/png", 0, 1 },
    };
    gint target_count = data->png_len > 0 ? 2 : 1;
    GtkClipboard *clipboard = gtk_clipboard_get(GDK_SELECTION_CLIPBOARD);
    if (!gtk_clipboard_set_with_data(
        clipboard,
        targets,
        target_count,
        lap_clipboard_get,
        lap_clipboard_clear,
        data
    )) {
        lap_clipboard_clear(NULL, data);
        return false;
    }
    gtk_clipboard_set_can_store(clipboard, targets, target_count);
    gtk_clipboard_store(clipboard);
    return true;
}
