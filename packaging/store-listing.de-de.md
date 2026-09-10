# Store listing text, German

The German half of `store-listing.md`, one file per language. The headings are
that file's headings and stay in English, because `fenster`'s parser reads both
files the same way; only what sits under them is German. Only the fields a store
shows a reader are here.

**Terminology is the application's own, out of `flyleaf/po/de.po`** — a listing that
calls a thing something the window does not teaches the customer a word the
product has no use for. Where the catalogue has a term, it wins.

**German runs longer than English.** Run `fenster/check-listing.ps1` on this file
after any edit to either language rather than trusting a translation to fit.
## Short description (Microsoft Store, 500)

Eine TOML-Datei als Baum bearbeiten, jeden Wert nach seiner Art, und sie so speichern, dass Kommentare, Schlüsselreihenfolge und Formatierung von allem, was Sie nicht bearbeitet haben, erhalten bleiben.

## App features (Microsoft Store, up to 20 bullets of 200 characters)

    Zeigt das Dokument als Baum, für jeden TOML-Typ eine eigene Darstellung, mit dem Quelltext daneben und hervorgehobener Zeile, in der Sie gerade arbeiten.
    Bearbeitet Werte nach ihrer Art — alle elf, die vier Datums- und Zeitformen auseinandergehalten — und wandelt einen Wert in eine andere Art um, wo er sich als diese lesen lässt.
    Fügt Schlüssel, Tabellen, Inline-Tabellen und Listen hinzu, benennt, wandelt und entfernt sie und bearbeitet Kommentare, wo TOML einen zulässt. Rückgängig und Wiederholen, ein Schritt je Zeile.
    Was Sie nicht bearbeitet haben, überlebt das Speichern: Kommentare, Schlüsselreihenfolge, Leerraum, Anführungszeichen. TOML 1.1.0; jeder gültige toml-test-Fall bleibt Byte für Byte gleich.
    Speichert atomar: die neue Datei ersetzt die alte in einem Schritt. Eine Unterbrechung hinterlässt die alte oder die neue Datei, nie halb beides. Schreibgeschützte Dateien werden abgelehnt.
    Keinerlei Netzwerkverbindung: kein Konto, keine Update-Prüfung, keine Analyse, Telemetrie oder Absturzberichte. Kein Konfigurationsverzeichnis, kein Verlauf, keine zuletzt geöffneten Dateien.

## Subtitle (Mac App Store, 30)

TOML-Editor, nur Ihre Änderung

## Promotional text (Mac App Store, 170)

TOML-Datei als Baum öffnen, einen Wert ändern, speichern. Jeder Kommentar und jede Zeile, die Sie nicht anfassen, kommt unverändert zurück. Nichts verlässt Ihren Mac.

## Description (both, written to 4,000)

Tommy Flyleaf ist ein TOML-Editor, der die Struktur der Datei kennt. Er öffnet eine Datei, zeigt sie als Baum neben dem, was ein Speichern schreiben würde, und bearbeitet jeden Wert nach seiner Art.

Ein Speichern behält jeden Kommentar, die Schlüsselreihenfolge, den Leerraum und die Anführungszeichen von allem, was Sie nicht bearbeitet haben. Eine Cargo.toml oder eine pyproject.toml kommt so zurück, wie Sie sie verlassen haben, mit einem geänderten Wert.

Was es tut:

• Zeigt das Dokument als Baum, für jeden TOML-Typ eine eigene Darstellung, mit dem Quelltext daneben und hervorgehobener Zeile, in der Sie gerade arbeiten.
• Bearbeitet Werte nach ihrer Art — alle elf, die vier Datums- und Zeitformen auseinandergehalten — und wandelt einen Wert in eine andere Art um, wo er sich als diese lesen lässt.
• Fügt Schlüssel, Tabellen, Inline-Tabellen und Listen hinzu, benennt sie um, wandelt sie um und entfernt sie.
• Bearbeitet Kommentare überall, wo TOML einen zulässt: über einem Schlüssel, neben einem Wert oder am Ende der Datei.
• Rückgängig und Wiederholen, ein Schritt je bearbeiteter Zeile.
• Speichert atomar. Die neue Datei ersetzt die alte in einem Schritt, sodass eine Unterbrechung die alte oder die neue Datei hinterlässt, nie halb beides. Eine schreibgeschützte Datei wird abgelehnt statt ersetzt.

TOML 1.1.0. Jeder gültige Fall des Konformitätskorpus toml-test kommt Byte für Byte unverändert zurück.

Tommy Flyleaf baut keinerlei Netzwerkverbindung auf: kein Konto, keine Update-Prüfung, keine Analyse, keine Telemetrie, keine Absturzberichte. Es schreibt kein Konfigurationsverzeichnis, keinen Verlauf und keine Liste zuletzt geöffneter Dateien.

Benannt nach einem Buchbinderlehrling. Das Vorsatzblatt ist die leere Seite hinter dem Buchdeckel, die einzige Stelle in einem gebundenen Buch, auf der später jemand schreiben soll. Das Buch gehört jemand anderem; schreiben Sie auf die vorgesehene Seite und lassen Sie den Rest so, wie er gebunden wurde.

Open Source, MIT-lizenziert: github.com/excelano/flyleaf

## Keywords

**Mac App Store** (100 characters, comma-separated, no spaces after commas):

    TOML,Editor,Konfiguration,Config,Cargo.toml,pyproject.toml,Entwickler,Einstellungen,verlustfrei

**Microsoft Store** (at most seven terms, each at most 30 characters):

    TOML, TOML-Editor, Konfigurationsdatei, Config-Editor, Cargo.toml, pyproject.toml, Entwicklerwerkzeuge

## Release notes

*Neu in dieser Version*, eine Fassung je Abschnitt, neueste zuerst. Der
Abschnitt unten trägt den Text der Mac-Einreichung, so wie im englischen
Original, weil das die Einreichung ist, die ansteht.

### 0.2.3

Tommy Flyleaf spricht Deutsch: auf einem deutsch eingestellten Rechner erscheint das Fenster auf Deutsch. Es ist auf einem skalierten Bildschirm scharf und hat ein eigenes Symbol.

### 0.2.1

Erste Veröffentlichung im Microsoft Store.
