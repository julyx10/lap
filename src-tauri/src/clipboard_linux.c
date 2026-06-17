#include <gtk/gtk.h>
#include <gio/gio.h>
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

unsigned char *lap_get_clipboard_file_paths(size_t *out_len) {
    if (out_len) *out_len = 0;

    GtkClipboard *clipboard = gtk_clipboard_get(GDK_SELECTION_CLIPBOARD);
    gchar **uris = gtk_clipboard_wait_for_uris(clipboard);
    if (!uris) return NULL;

    GString *paths = g_string_new(NULL);
    for (gchar **uri = uris; *uri; uri++) {
        GFile *file = g_file_new_for_uri(*uri);
        if (!file) continue;

        gchar *path = g_file_get_path(file);
        if (path && *path) {
            if (paths->len > 0) g_string_append_c(paths, '\n');
            g_string_append(paths, path);
        }

        g_free(path);
        g_object_unref(file);
    }
    g_strfreev(uris);

    if (paths->len == 0) {
        g_string_free(paths, TRUE);
        return NULL;
    }

    if (out_len) *out_len = paths->len;
    return (unsigned char *)g_string_free(paths, FALSE);
}

unsigned char *lap_get_clipboard_png(size_t *out_len) {
    if (out_len) *out_len = 0;

    GtkClipboard *clipboard = gtk_clipboard_get(GDK_SELECTION_CLIPBOARD);
    GdkPixbuf *pixbuf = gtk_clipboard_wait_for_image(clipboard);
    if (!pixbuf) return NULL;

    gchar *buffer = NULL;
    gsize buffer_len = 0;
    GError *error = NULL;
    gboolean ok = gdk_pixbuf_save_to_buffer(
        pixbuf,
        &buffer,
        &buffer_len,
        "png",
        &error,
        NULL
    );
    g_object_unref(pixbuf);

    if (!ok || !buffer || buffer_len == 0) {
        if (error) g_error_free(error);
        g_free(buffer);
        return NULL;
    }

    if (out_len) *out_len = (size_t)buffer_len;
    return (unsigned char *)buffer;
}

void lap_clipboard_free(void *ptr) {
    g_free(ptr);
}
