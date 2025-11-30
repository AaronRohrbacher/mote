#include <wayland-client.h>
#include <gtk/gtk.h>
#include <stdlib.h>

typedef struct {
    GtkWidget *window;
    GtkWidget *button;
    char *command;
} Icon;

static void on_click(GtkWidget *widget, gpointer data) {
    Icon *icon = (Icon *)data;
    system(icon->command);
}

static GtkWidget *create_icon(const char *name, const char *emoji, const char *command, int x, int y) {
    Icon *icon = g_malloc(sizeof(Icon));
    icon->command = g_strdup(command);
    
    icon->window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_decorated(GTK_WINDOW(icon->window), FALSE);
    gtk_window_set_skip_taskbar_hint(GTK_WINDOW(icon->window), TRUE);
    gtk_window_set_keep_above(GTK_WINDOW(icon->window), TRUE);
    gtk_window_set_default_size(GTK_WINDOW(icon->window), 150, 150);
    gtk_window_move(GTK_WINDOW(icon->window), x, y);
    gtk_window_stick(GTK_WINDOW(icon->window));
    
    icon->button = gtk_button_new();
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    
    GtkWidget *label_emoji = gtk_label_new(NULL);
    char *emoji_markup = g_strdup_printf("<span font='Sans 72'>%s</span>", emoji);
    gtk_label_set_markup(GTK_LABEL(label_emoji), emoji_markup);
    g_free(emoji_markup);
    
    GtkWidget *label_name = gtk_label_new(NULL);
    char *name_markup = g_strdup_printf("<span font='Sans Bold 16'>%s</span>", name);
    gtk_label_set_markup(GTK_LABEL(label_name), name_markup);
    g_free(name_markup);
    
    gtk_box_pack_start(GTK_BOX(box), label_emoji, TRUE, TRUE, 0);
    gtk_box_pack_start(GTK_BOX(box), label_name, TRUE, TRUE, 0);
    gtk_container_add(GTK_CONTAINER(icon->button), box);
    gtk_container_add(GTK_CONTAINER(icon->window), icon->button);
    
    g_signal_connect(icon->button, "clicked", G_CALLBACK(on_click), icon);
    
    gtk_widget_show_all(icon->window);
    return icon->window;
}

int main(int argc, char *argv[]) {
    gtk_init(&argc, &argv);
    
    create_icon("Mote", "🖥️", 
        "ssvncviewer -quality 9 -compresslevel 0 -fullscreen -scale '800x480' 10.1.1.79 & sleep 1 && /Users/aaronr/mote/home-button.sh &",
        50, 50);
    create_icon("Chromium", "🌐", "cromium &", 230, 50);
    
    gtk_main();
    return 0;
}

