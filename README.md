# ctrlb

ctrlb speichert alles, was du als Text kopierst, und lässt dich ältere Kopien mit einem kleinen Menü wieder einfügen.

## Benutzung

1. Starte `ctrlb.exe`. Es gibt kein Fenster. Das Programm läuft als kleines graues Symbol unten rechts in der Taskleiste neben der Uhr (eventuell hinter dem Pfeil ^).
2. Kopiere wie gewohnt Text (Ctrl+C). Er wird automatisch gespeichert.
3. Halte **Ctrl** gedrückt und drücke **B** (mit oder ohne **Shift**). In der Bildschirmmitte erscheint das Menü mit der letzten Kopie.
   - **Ctrl+B**: nächste ältere Kopie
   - **Ctrl+Shift+B**: zurück zur nächsten neueren Kopie
   - Am Ende der Liste springt die Auswahl auf die andere Seite.
4. Lass **Ctrl** los. Das Menü verschwindet und der gewählte Text liegt in der Zwischenablage.
5. Füge ihn mit Ctrl+V ein.

## Beenden

Klicke auf das Symbol in der Taskleiste und wähle **Quit**.

## Gespeicherte Daten

- Die Kopien stehen in der Datei `clipboard_history.txt`, die neben `ctrlb.exe` liegt.
- Es werden die letzten 50 Texte gespeichert. Ältere fallen weg.
- Wird derselbe Text nochmal kopiert, rutscht er nach vorne und steht nicht doppelt in der Liste.
- Nur Text wird gespeichert, keine Bilder oder Dateien.
- Zum Löschen der Liste: Programm beenden und `clipboard_history.txt` löschen.

## Bei jedem Windows-Start automatisch starten

1. Lege `ctrlb.exe` in einen festen Ordner, z. B. `C:\Tools\ctrlb\`. Die Datei `clipboard_history.txt` wird dort gespeichert.
2. Drücke Win+R, tippe `shell:startup` und drücke Enter.
3. Erstelle in diesem Ordner eine Verknüpfung zur `ctrlb.exe` (Rechtsklick auf die .exe, Verknüpfung erstellen, die Verknüpfung in den Startup-Ordner schieben).

## Gut zu wissen

- Ctrl+B und Ctrl+Shift+B haben in manchen Programmen eigene Funktionen (z. B. Fett oder die Lesezeichenleiste im Browser). Sie werden dort trotzdem ausgelöst, weil ctrlb die Tasten nur mitliest.
- Es dürfen keine zwei Kopien von ctrlb gleichzeitig laufen.

## Dateien des Projekts

```
ctrlb/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs                     Start
    ├── app.rs                      Verbindet Menü, Tray-Symbol und Logik
    ├── theme.rs                    Farben, Größen, Limits, Zeiten
    ├── ui.rs
    ├── ui/
    │   ├── overlay_view.rs         Zeichnet das Menü
    │   └── tray_menu.rs            Tray-Symbol mit "Quit"
    ├── logic.rs
    ├── logic/
    │   ├── history_list.rs         Liste der Kopien
    │   ├── history_file.rs         Datei laden und speichern
    │   ├── menu_selection.rs       Welcher Eintrag ist gewählt
    │   ├── clipboard_watcher.rs    Zwischenablage beobachten
    │   └── hotkey_watcher.rs       Ctrl+B, Ctrl+Shift+B und Ctrl-Loslassen erkennen
    ├── utils.rs
    └── utils/
        └── text_format.rs          Text für das Menü kürzen
```
