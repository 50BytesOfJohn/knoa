## Terms
## One word per concept, used everywhere in this language.

-knoa = Knoa
-workspace = Arbeitsbereich
-bookmark = Lesezeichen
-folder = Ordner
-tag = Tag
-summary = Zusammenfassung
-organize = ordnen

## Menus

menu-about = Über { $name }
menu-check-for-updates = Nach Updates suchen…
menu-settings = Einstellungen…
menu-services = Dienste
menu-hide = { $name } ausblenden
menu-hide-others = Andere ausblenden
menu-show-all = Alle einblenden
menu-quit = { $name } beenden
menu-help = Hilfe
menu-whats-new = Neu in { $name } { $version }
menu-report-issue = Problem melden…

## Onboarding

onboarding-skip-setup = Einrichtung überspringen →
onboarding-back = Zurück
onboarding-next = Weiter
onboarding-hello-title = Hi, ich bin { -knoa }.
onboarding-hello-lead = Du sammelst Lesezeichen wie ein Eichhörnchen Vorräte für den Winter. Ich finde sie wieder. Ich lese deine Browser-Lesezeichen, finde heraus, was dahintersteckt, und halte alles ordentlich.
onboarding-hello-questions = Drei kurze Fragen. Oder keine, wenn du es eilig hast.
onboarding-hello-start = Los geht’s
onboarding-hello-defaults = Standardwerte verwenden
onboarding-hello-have-account = Schon ein { -knoa }-Konto?
onboarding-hello-sign-in = Anmelden
onboarding-browser-title = Wo wohnen deine Lesezeichen?
onboarding-browser-lead = Speichere Lesezeichen einfach weiter wie gewohnt. Neue bemerke ich in wenigen Sekunden. Ich lese nur mit und fasse deine Browser-Lesezeichen nie an.
onboarding-browser-count =
    { $count ->
        [one] { $count } Lesezeichen
       *[other] { $count } Lesezeichen
    }
onboarding-browser-none-found = Ich habe hier keinen Chromium-Browser gefunden (Helium, Chrome, Brave, Edge, Vivaldi). Kein Problem: Du kannst später in den Einstellungen einen wählen.
onboarding-browser-bring-in = { $count } übernehmen
onboarding-ai-title = Soll ich das Aufräumen übernehmen?
onboarding-ai-lead = Mit einem OpenRouter-Schlüssel lese ich jede neue Seite und vergebe Art, Zusammenfassung, Tags und Ordner. Es ist dein Schlüssel und dein Konto: etwa 17 Cent pro tausend Lesezeichen.
onboarding-ai-key-placeholder = Schlüssel einfügen: sk-or-v1-…
onboarding-ai-check-key = Schlüssel prüfen
onboarding-ai-checking = Klopfe bei OpenRouter an…
onboarding-ai-key-works = Der Schlüssel funktioniert.
onboarding-ai-key-missing = Füge zuerst einen Schlüssel ein oder überspringe diesen Schritt.
onboarding-ai-key-unknown = OpenRouter kennt diesen Schlüssel nicht. Prüfst du ihn nochmal?
onboarding-ai-key-check-failed = Schlüssel konnte nicht geprüft werden: { $error }
onboarding-ai-get-key = Schlüssel holen ↗
onboarding-ai-without-key = Ohne Schlüssel hole ich trotzdem Bilder und Beschreibungen.
onboarding-ai-skip = Überspringen, ohne KI
onboarding-library-title = Letzte Frage: Wie soll diese Bibliothek heißen?
onboarding-library-default-name = Privat
onboarding-library-house-rules = Gibt es Hausregeln fürs Ordnen?
onboarding-library-instructions-placeholder = Optional. Schreib es so, wie du es Freunden erklären würdest.
onboarding-library-example-apart = Berufliches und Privates trennen.
onboarding-library-example-recipes = Rezepte nach Küche taggen.
onboarding-library-example-rust = Alles über Rust kommt in einen Rust-Ordner.
onboarding-library-example-short = Kurze Zusammenfassungen, ohne Schnickschnack.
onboarding-library-add-example = + { $example }
onboarding-library-finish = Fertig
onboarding-done-title = Alles bereit.
onboarding-done-no-browser = Noch kein Browser gewählt, deine Bibliothek startet also leer. Du kannst jederzeit einen in den Einstellungen wählen.
onboarding-done-bringing-in = Deine Lesezeichen werden übernommen…
onboarding-done-count =
    { $count ->
        [one] { $count } Lesezeichen ist da und sofort durchsuchbar.
       *[other] { $count } Lesezeichen sind da und sofort durchsuchbar.
    }
onboarding-done-organize-existing =
    { $count ->
        [one] Neue Lesezeichen werden von selbst geordnet. Soll ich das vorhandene Lesezeichen auch durchgehen? Etwa { $cost }.
       *[other] Neue Lesezeichen werden von selbst geordnet. Soll ich die { $count } vorhandenen auch durchgehen? Etwa { $cost }.
    }
onboarding-done-organize = Ja, ordnen
onboarding-done-open-library = Meine Bibliothek öffnen

## Settings

settings-title = Einstellungen
settings-workspaces = Arbeitsbereiche
settings-new-workspace = Neuer Arbeitsbereich
settings-nav-general = Allgemein
settings-nav-appearance = Erscheinungsbild
settings-nav-ai = KI
settings-nav-account = Konto
settings-nav-organize = Ordnen
settings-nav-mcp = MCP
settings-general-title = Allgemein
settings-general-about = Wo { -knoa } deine Bibliothek ablegt, wie sie eingerichtet wurde und wie sie sich aktualisiert.
settings-general-library = Bibliothek
settings-general-location = Speicherort der Bibliothek
settings-general-show-in-finder = Im Finder anzeigen
settings-general-setup = Einrichtung
settings-general-run-setup-again = Einrichtung erneut starten
settings-general-run-setup-about = Browser, KI und Anweisungen neu wählen. Deine Lesezeichen bleiben erhalten.
settings-general-run-setup = Einrichtung starten
settings-general-language = Sprache
settings-general-language-label = App- und KI-Sprache
settings-general-language-about = Menüs, Einstellungen und Datumsangaben sowie die Zusammenfassungen und Tags der KI.
settings-general-language-note = Bereits erstellte Zusammenfassungen und Tags bleiben in ihrer alten Sprache. Durch erneutes Ordnen deiner Bibliothek werden sie neu geschrieben.
settings-general-language-organize = Erneut ordnen…
settings-general-language-organize-confirm = Alle ordnen
settings-general-language-waiting = Du kannst erneut ordnen, sobald dein Konto die neue Sprache hat.
settings-updates = Updates
settings-updates-version = { -knoa } { $version }
settings-updates-self-built = Dieser Build aktualisiert sich nicht selbst. Release-Builds von GitHub schon.
settings-updates-checked-today = Zuletzt heute um { $time } geprüft
settings-updates-checked-yesterday = Zuletzt gestern um { $time } geprüft
settings-updates-checked-on = Zuletzt am { $date } geprüft
settings-updates-not-checked = Noch nicht geprüft
settings-updates-check-now = Jetzt suchen
settings-updates-check-automatically = Automatisch nach Updates suchen
settings-updates-check-automatically-about = Einmal täglich im Hintergrund.
settings-updates-install-automatically = Updates automatisch installieren
settings-updates-install-automatically-about = Neue Versionen laden und installieren, wenn du { -knoa } beendest.
settings-appearance-title = Erscheinungsbild
settings-appearance-about = Wie { -knoa } aussieht. Änderungen sind sofort sichtbar.
settings-appearance-theme = Design
settings-appearance-light = Hell
settings-appearance-dark = Dunkel
settings-appearance-library = Bibliothek
settings-appearance-summaries = Zusammenfassungen auf Karten anzeigen
settings-appearance-summaries-about = Bis zu drei Zeilen unter jedem Titel.
settings-appearance-up-next = Leiste „Als Nächstes“ anzeigen
settings-appearance-up-next-about = Die Reihe zum Lesen oder Anschauen ganz oben unter „Alles“.
settings-appearance-events = Events anzeigen als
settings-appearance-events-about = Als Monatskalender mit kommenden Terminen oder als Liste. Nur auf diesem Mac.
settings-reorganize-title = Erneut ordnen
settings-reorganize-folders-confirm = Alle einsortieren
settings-reorganize-everything-confirm = Alle ordnen
settings-organize-title = Ordnen
settings-organize-about = Lass die KI Lesezeichen ordnen, zu denen sie noch nicht gekommen ist, oder die bereits geordneten noch einmal durchgehen.
settings-organize-scope = Lesezeichen in
settings-organize-scope-all = Allen Arbeitsbereichen
settings-organize-new = Noch nicht geordnet
settings-organize-counting = Lesezeichen werden gezählt…
settings-organize-none = Hier ist alles geordnet.
settings-organize-waiting =
    { $count ->
        [one] { $count } Lesezeichen ist noch nicht geordnet
       *[other] { $count } Lesezeichen sind noch nicht geordnet
    }
settings-organize-ai-off = Ordnen braucht KI
settings-organize-ai-off-about = Verbinde einen eigenen Schlüssel oder hol dir Pro auf der KI-Seite.
settings-organize-open-ai = KI einrichten
settings-reorganize-folders-label = Nur Ordner
settings-reorganize-folders-about = Sortiert geordnete Lesezeichen in deine jetzigen Ordner ein. Seiten werden nicht neu gelesen, das geht schnell und kostet wenig.
settings-reorganize-folders-start = Neu einsortieren…
settings-reorganize-everything-label = Alles
settings-reorganize-everything-about = Liest jede Seite neu und erstellt Zusammenfassung, Felder, Tags und Ordner mit den aktuellen Anweisungen und der aktuellen Sprache neu.
settings-reorganize-everything-start = Erneut ordnen…
settings-reorganize-kept = Ordner, die du selbst gesetzt oder entfernt hast, bleiben, wie sie sind.
settings-reorganize-queued =
    { $count ->
        [one] { $count } Lesezeichen eingereiht.
       *[other] { $count } Lesezeichen eingereiht.
    }
settings-account-synced-today = Zuletzt heute um { $time } synchronisiert
settings-account-synced-yesterday = Zuletzt gestern um { $time } synchronisiert
settings-account-synced-on = Zuletzt am { $date } synchronisiert
settings-account-not-synced = Noch nicht synchronisiert
settings-account-start-trial = Kostenlos testen
settings-account-upgrade = Upgrade
settings-account-manage-subscription = Abo verwalten
settings-account-usage = Nutzung
settings-account-usage-device = { $used } von { $limit } Lesezeichen
settings-account-usage-cloud = { $used } von { $limit } Lesezeichen, { $organized } von { $allowance } diesen Monat geordnet
settings-account-plan-free = Kostenloser Tarif.
settings-account-plan-trial = Pro-Testphase.
settings-account-plan-pro = Pro-Tarif.
settings-account-plan-ends = Endet am { $date }.
settings-account-plan-trial-ends = Testphase endet am { $date }.
settings-account-plan-renews = Verlängert sich am { $date }.
settings-account-organized-on-device = Geordnet wird mit deinem eigenen KI-Schlüssel auf diesem Mac.
settings-account-organized-in-cloud = { -knoa } ordnet deine Lesezeichen in der Cloud.
settings-account-title = Konto
settings-account-about = Synchronisiere deine Bibliothek über ein { -knoa }-Konto.
settings-account-unavailable = Cloud-Sync ist in diesem Build nicht verfügbar.
settings-account-couldnt-start = Konnte nicht gestartet werden: { $reason }
settings-account-cloud-mode = Cloud-Modus
settings-account-what-it-is = Im Cloud-Modus wird deine Bibliothek auf deinen Geräten und mit { -knoa } im Web synchronisiert. Mit Pro ordnet { -knoa } deine Lesezeichen auch in der Cloud – ganz ohne eigenen Schlüssel.
settings-account-privacy = Im Cloud-Modus liegt deine Bibliothek auf den Servern von { -knoa }, die deine Seiten zum Ordnen lesen. Sie ist nicht Ende-zu-Ende-verschlüsselt. Abgemeldet bleibt alles auf diesem Mac.
settings-account-finish-signing-in = Anmeldung im Browser abschließen…
settings-account-finish-signing-in-about = { -knoa } hat die Anmeldeseite in deinem Browser geöffnet.
settings-account-cancel = Abbrechen
settings-account-knoa-account = { -knoa }-Konto
settings-account-sign-in-about = Melde dich im Browser an oder erstelle ein Konto.
settings-account-sign-in = Anmelden
settings-account-other-account = Diese Bibliothek ist mit einem anderen { -knoa }-Konto synchronisiert. Solange du nichts wählst, wird nicht synchronisiert.
settings-account-sync-with = Stattdessen mit { $email } synchronisieren
settings-account-sync-with-this-account = Stattdessen mit diesem Konto synchronisieren
settings-account-sync-with-about = Die Bibliothek wird mit diesem Konto zusammengeführt. Das andere Konto behält seine Kopie.
settings-account-use-this-account = Dieses Konto verwenden
settings-account-this-account = Dieses Konto
settings-account-asking = Angemeldet. Kontodaten werden abgerufen…
settings-account-sync = Sync
settings-account-connecting = Wird verbunden…
settings-account-up-to-date = Aktuell
settings-account-syncing =
    { $count ->
        [one] { $count } Änderung wird synchronisiert…
       *[other] { $count } Änderungen werden synchronisiert…
    }
settings-account-offline = Offline. Änderungen werden synchronisiert, sobald { -knoa } den Server erreicht.
settings-account-sync-stopped = Sync gestoppt: { $error }
settings-account-notice-plan-limit =
    { $unsent ->
        [one] Dein Tarif umfasst { $limit } Lesezeichen, daher bleibt { $unsent } neues Lesezeichen nur auf diesem Mac.
       *[other] Dein Tarif umfasst { $limit } Lesezeichen, daher bleiben { $unsent } neue Lesezeichen nur auf diesem Mac.
    }
settings-account-notice-rejected = Einige Änderungen konnten nicht synchronisiert werden und bleiben nur auf diesem Mac.
settings-account-notice-too-large = Einige Änderungen waren zu groß zum Synchronisieren und bleiben nur auf diesem Mac.
settings-account-notice-allowance-used-up = Das Kontingent zum Ordnen für diesen Monat ist aufgebraucht. { -knoa } merkt sich deine Anfragen und macht automatisch weiter, sobald wieder Kontingent frei ist.
settings-account-sign-out = Abmelden
settings-account-sign-out-about = Deine Bibliothek bleibt auf diesem Mac. Melde dich wieder an, um weiter zu synchronisieren.
settings-ai-access-about = Lass KI-Apps wie Claude und ChatGPT deine Lesezeichen lesen und verwalten.
settings-ai-access-copy = Kopieren
settings-ai-access-url = Connector-URL
settings-ai-access-default = Standardzugriff
settings-ai-access-same-as-default = Wie Standard
settings-ai-access-off = Aus
settings-ai-access-read = Nur lesen
settings-ai-access-full = Vollzugriff
settings-ai-access-not-linked = Verfügbar, sobald dieser Arbeitsbereich mit der Cloud verbunden ist.
settings-ai-access-saving = Wird gespeichert…
settings-ai-access-loading = KI-Zugriff wird geladen…
settings-ai-access-save-again = KI-Zugriff erneut speichern
settings-ai-access-load-again = KI-Zugriff erneut laden
settings-ai-access-retry = Nochmal
settings-ai-access-needs-pro = KI-Zugriff erfordert { -knoa } Pro
settings-mcp-title = MCP
settings-mcp-sign-in = Melde dich an, um KI-Apps zu verbinden
settings-mcp-sign-in-about = KI-Apps erreichen deine Bibliothek über dein { -knoa }-Konto.
settings-mcp-open-account = Konto öffnen
settings-mcp-pro-about = Mit Pro können KI-Apps wie Claude und ChatGPT deine Bibliothek durchsuchen, lesen und ordnen.
settings-mcp-copied = Kopiert
settings-mcp-connect = Verbinden
settings-mcp-how = So verbindest du dich
settings-mcp-how-copy = Kopiere die Connector-URL.
settings-mcp-how-add = Füge sie in Claude, ChatGPT oder einer anderen KI-App als eigenen Connector hinzu.
settings-mcp-how-sign-in = Melde dich bei { -knoa } an, wenn die App danach fragt.
settings-mcp-how-about = Eine verbundene KI-App kann deine Lesezeichen inklusive gespeicherter Seitentexte in jedem freigegebenen Arbeitsbereich lesen. Änderungen hier gelten ab der nächsten Anfrage.
settings-mcp-access = Zugriff
settings-mcp-default-off = KI-Apps haben keinen Zugriff auf Arbeitsbereiche mit Standardeinstellung.
settings-mcp-default-read = KI-Apps können Lesezeichen in Arbeitsbereichen mit Standardeinstellung durchsuchen und lesen.
settings-mcp-default-full = KI-Apps können außerdem Lesezeichen hinzufügen, bearbeiten und in den Papierkorb legen sowie Ordner und Arbeitsbereiche verwalten.
settings-ai-title = KI
settings-ai-about = { -knoa } funktioniert auch ohne KI. Mit ihr bekommen neue Lesezeichen Art, Zusammenfassung, Tags und Ordner.
settings-ai-provider = Anbieter
settings-ai-cloud-note = Mit Pro ordnet { -knoa } deine Lesezeichen in der Cloud. Ein verbundener Schlüssel bleibt auf diesem Mac, falls du nicht angemeldet bist.
settings-ai-key = OpenRouter-Schlüssel
settings-ai-key-about = Dein eigener Schlüssel; Anfragen gehen von diesem Mac direkt an OpenRouter.
settings-ai-connect = Verbinden
settings-ai-checking = Schlüssel wird geprüft…
settings-ai-key-connected = Verbunden. Neue Lesezeichen werden ab jetzt geordnet.
settings-ai-key-failed = Das hat nicht geklappt: { $error }.
settings-ai-no-key = Noch kein Schlüssel?
settings-ai-no-key-about = Ein paar Dollar Guthaben reichen für tausende Lesezeichen.
settings-ai-get-key = Schlüssel holen
settings-ai-connected = Verbunden
settings-ai-disconnect = Trennen
settings-ai-openrouter-says = OpenRouter meldet: { $problem }. Prüfe deinen Schlüssel oder dein Guthaben.
settings-ai-models = Modelle
settings-ai-included-in-pro = In Pro enthalten, im Rahmen des monatlichen Kontingents.
settings-ai-cost = Etwa { $cost } für alle.
settings-ai-organize = Ordnen
settings-ai-pro-title = Lass { -knoa } das mit Pro erledigen
settings-ai-pro-pitch = { -knoa } ordnet deine Lesezeichen in der Cloud. Kein Schlüssel einzurichten, keine Abrechnung pro Anfrage, und deine Bibliothek synchronisiert sich auf deinen Geräten.
settings-ai-pro-get = Pro holen
settings-ai-pro-upgrade = Upgrade
settings-ai-pro-have = Ich habe Pro
settings-ai-usage = Nutzung
settings-ai-usage-none = Noch keine KI-Anfragen.
settings-ai-usage-recent =
    { $days ->
        [one] Letzter Tag
       *[other] Letzte { $days } Tage
    }
settings-ai-usage-all-time = Gesamt
settings-ai-usage-by-model =
    { $days ->
        [one] Nach Modell, letzter Tag
       *[other] Nach Modell, letzte { $days } Tage
    }
settings-ai-usage-records = Aufzeichnungen von { $provider }
settings-ai-usage-records-about = Jede Anfrage mit dem genauen abgerechneten Betrag.
settings-ai-usage-open = Öffnen
settings-ai-usage-requests =
    { $count ->
        [one] { $count } Anfrage
       *[other] { $count } Anfragen
    }
settings-ai-usage-tokens = { $input } Tokens rein, { $output } raus
settings-ai-usage-unmetered =
    { $count ->
        [one] { $count } ohne Token-Angabe
       *[other] { $count } ohne Token-Angabe
    }
settings-ai-usage-unpriced =
    { $count ->
        [one] { $count } ohne Preis
       *[other] { $count } ohne Preis
    }
settings-workspace-new = Neuer Arbeitsbereich
settings-workspace-new-about = Eine eigene Bibliothek, etwa für Arbeit oder Privates, mit eigenen Browsern und Ordnern.
settings-workspace-shown-now = Wird angezeigt
settings-workspace-show-it = Anzeigen
settings-workspace-name-and-color = Name und Farbe
settings-workspace-name = Name
settings-workspace-name-too-long = Ein Name darf höchstens { $max } Zeichen lang sein.
settings-workspace-color = Farbe
settings-workspace-no-browser = Kein Chromium-Browser gefunden. { -knoa } liest Helium, Chrome, Brave, Edge, Vivaldi und Chromium.
settings-workspace-browsers-note = Ist keiner aktiv, werden Links nur von Hand hinzugefügt (⌘N). Ein Browser kann mehrere Arbeitsbereiche versorgen.
settings-workspace-open-in = Lesezeichen öffnen in
settings-workspace-browsers = Browser
settings-workspace-open-in-about = Wo sich die Links dieses Arbeitsbereichs öffnen.
settings-workspace-organize-again-about = Wenn du Anweisungen oder Tags änderst, sortiere oder ordne die Lesezeichen dieses Arbeitsbereichs neu.
settings-workspace-open-organize = Ordnen…
settings-workspace-default-browser = Standardbrowser
settings-workspace-by-hand = Nur von Hand hinzugefügte Links.
settings-workspace-synced-from-one = Synchronisiert aus { $browser }.
settings-workspace-synced-from-many = Synchronisiert aus { $browsers } und { $last }.
settings-workspace-organizing = Ordnen
settings-workspace-instructions = Anweisungen
settings-workspace-instructions-about = Wie die KI sortieren, taggen und zusammenfassen soll, was hier landet.
settings-workspace-instructions-too-long = Anweisungen dürfen höchstens { $max } Zeichen lang sein.
settings-workspace-screening = Nur passende Lesezeichen behalten
settings-workspace-screening-about = Aus: Alles aus dem Browser landet hier. Ein: Die KI stellt Unpassendes zurück, und du kannst es trotzdem behalten.
settings-workspace-guideline = Was hier hineingehört
settings-workspace-guideline-too-long = Eine Richtlinie darf höchstens { $max } Zeichen lang sein.
settings-workspace-name-placeholder = Arbeit, Privat…
settings-workspace-instructions-placeholder = z. B. Rezepte nach Küche taggen. Kurze Zusammenfassungen, ohne Schnickschnack.
settings-workspace-guideline-placeholder = z. B. Nur Arbeit: Entwicklung, Design und unsere Kunden.
settings-workspace-create = Arbeitsbereich erstellen
settings-workspace-delete = Löschen
settings-workspace-delete-label = Diesen Arbeitsbereich löschen
settings-workspace-delete-about = Seine Lesezeichen und Ordner werden mitgelöscht. Deine Browser behalten ihre Lesezeichen.
settings-workspace-delete-button = Löschen…
settings-workspace-delete-confirm = Endgültig löschen
settings-tags-title = Tag-Liste
settings-tags-label = Tags, die die KI nutzen soll
settings-tags-about = Passt einer davon zu einem Lesezeichen, nimmt die KI ihn statt eines eigenen Begriffs. Ein kurzer Hinweis erklärt, wann er passt. Du kannst einen Tag an einem Lesezeichen auch per Rechtsklick aufnehmen.
settings-tags-add-placeholder = Tag hinzufügen
settings-tags-hint-placeholder = Wann verwenden (optional)
settings-tags-remove = Aus Liste entfernen
settings-tags-only-listed = Nur Tags aus dieser Liste verwenden
settings-tags-only-listed-about = Aus: Die KI ergänzt eigene Tags für alles, was nicht in der Liste steht. Ein: Lesezeichen bekommen nur Tags aus der Liste, oder keine, wenn nichts passt.
settings-tags-only-listed-about-empty = Füge zuerst einen Tag hinzu. Aus: Die KI ergänzt eigene Tags für alles, was nicht in der Liste steht. Ein: Lesezeichen bekommen nur Tags aus der Liste, oder keine, wenn nichts passt.
settings-tags-note = Ab jetzt geordnete Lesezeichen bekommen diese Tags. Für bereits geordnete einfach erneut ordnen.
settings-tags-count = { $count } von { $max }

## Sidebar

sidebar-status-ai-needs-attention = KI braucht Aufmerksamkeit
sidebar-status-fix = Beheben
sidebar-sync-stopped = Sync gestoppt
sidebar-retry = Nochmal
sidebar-sync-paused = Sync pausiert
sidebar-status-changes-not-synced = Nicht synchronisiert
sidebar-status-view = Ansehen
sidebar-status-plan-limit = Tariflimit erreicht
sidebar-status-upgrade = Upgrade
sidebar-status-organizing = Ordnet { $current } von { $total }…
sidebar-status-ai-off = KI ist aus
sidebar-status-turn-on = Einschalten
sidebar-status-offline = Offline. Sync folgt später.
sidebar-syncing = Wird synchronisiert…
sidebar-sync-offline =
    { $count ->
        [0] Offline
        [one] Offline · { $count } Änderung ausstehend
       *[other] Offline · { $count } Änderungen ausstehend
    }
sidebar-synced-just-now = Gerade synchronisiert
sidebar-synced-minutes-ago =
    { $minutes ->
        [one] Vor { $minutes } Min. synchronisiert
       *[other] Vor { $minutes } Min. synchronisiert
    }
sidebar-synced-hours-ago =
    { $hours ->
        [one] Vor { $hours } Std. synchronisiert
       *[other] Vor { $hours } Std. synchronisiert
    }
sidebar-account-settings = Kontoeinstellungen…
sidebar-open-web = { -knoa } im Web öffnen
sidebar-sign-out = Abmelden
sidebar-sign-in = Anmelden…
sidebar-finish-in-browser = Im Browser abschließen
sidebar-cancel = Abbrechen
sidebar-plan-pro = { -knoa } Pro
sidebar-plan-free = Kostenloser Tarif
sidebar-signed-in = Angemeldet
sidebar-manage-plan = Tarif verwalten…
sidebar-upgrade-to-pro = Auf Pro upgraden
sidebar-this-mac = Dieser Mac
sidebar-knoa-account = { -knoa }-Konto
sidebar-place-everything = Alles
sidebar-place-gallery = Galerie
sidebar-place-events = Events
sidebar-place-up-next = Als Nächstes
sidebar-place-unsorted = Ohne Ordner
sidebar-place-not-organized = Nicht geordnet
sidebar-place-set-aside = Zurückgestellt
sidebar-place-trash = Papierkorb
sidebar-settings = Einstellungen
sidebar-folders = Ordner
sidebar-new-folder = Neuer Ordner
sidebar-folder-new-subfolder = Neuer Unterordner
sidebar-folder-edit = Ordner bearbeiten…
sidebar-folder-delete = Löschen…
sidebar-folder-move-to = Verschieben nach
sidebar-folder-top-level = Oberste Ebene
sidebar-add-link = Link hinzufügen
sidebar-workspaces-title = Arbeitsbereiche
sidebar-workspaces-new = Neuer Arbeitsbereich
sidebar-workspaces-by-hand = Links von Hand hinzugefügt
sidebar-workspaces-from = Aus { $browsers }

## Main

main-blocked-browsers =
    { $count ->
        [one] { $browsers } ist installiert, aber macOS schützt dessen Lesezeichen. Gib { -knoa } Festplattenvollzugriff und starte { -knoa } neu.
       *[other] { $browsers } sind installiert, aber macOS schützt ihre Lesezeichen. Gib { -knoa } Festplattenvollzugriff und starte { -knoa } neu.
    }
main-open-privacy-settings = Datenschutzeinstellungen öffnen
main-cancel = Abbrechen
main-delete = Löschen
main-save = Speichern
main-folder-new-title = Neuer Ordner
main-folder-edit-title = Ordner bearbeiten
main-folder-name-placeholder = Ordnername
main-folder-description-placeholder = Was gehört hier hinein? Danach sortiert die KI Lesezeichen ein.
main-folder-name-too-long = Ein Name darf höchstens { $max } Zeichen lang sein.
main-folder-description-too-long = Eine Beschreibung darf höchstens { $max } Zeichen lang sein.
main-folder-auto-assign = KI darf Lesezeichen hinzufügen
main-folder-auto-assign-about = Passende neue Lesezeichen landen von selbst hier.
main-glyph-read = Lesen
main-glyph-article = Artikel
main-glyph-code = Code
main-glyph-write = Schreiben
main-glyph-work = Arbeit
main-glyph-tag = Angebote
main-glyph-bag = Einkaufen
main-glyph-plane = Reisen
main-glyph-cup = Essen
main-glyph-music = Musik
main-glyph-play = Video
main-glyph-photo = Fotos
main-glyph-bulb = Ideen
main-glyph-heart = Gesundheit
main-glyph-home = Zuhause
main-glyph-star = Favoriten
main-glyph-clock = Später
main-glyph-flask = Wissenschaft
main-glyph-globe = Web
main-glyph-person = Personen
main-glyph-box = Archiv
main-glyph-inbox = Eingang
main-glyph-leaf = Natur
main-glyph-bolt = Schnell
main-glyph-lock = Privat
main-glyph-pin = Orte
main-glyph-chat = Threads
main-glyph-game = Spiele
main-folder-icon-none = Ordner
main-folder-icon-emoji = Emoji
main-folder-parent = Übergeordnet
main-folder-parent-none = Keiner
main-folder-emoji-placeholder = Tippen oder einfügen
main-folder-any-emoji = Beliebiges Emoji
main-folder-custom-color = Eigene Farbe
main-delete-folder-title = „{ $name }“ löschen?
main-delete-folder-detail =
    { $count ->
        [0] Die Lesezeichen bleiben in deiner Bibliothek.
        [one] Der Unterordner wird mitgelöscht. Die Lesezeichen bleiben in deiner Bibliothek.
       *[other] Die { $count } Unterordner werden mitgelöscht. Die Lesezeichen bleiben in deiner Bibliothek.
    }
main-open = Öffnen
main-open-in-background = Im Hintergrund öffnen
main-copy-link = Link kopieren
main-copied = Kopiert
main-show-details = Details anzeigen
main-view-picture = Bild ansehen
main-copy-image = Bild kopieren
main-save-image = Bild speichern…
main-add-to-up-next = Zu „Als Nächstes“ hinzufügen
main-remove-from-up-next = Aus „Als Nächstes“ entfernen
main-add-to-folder = Zum Ordner hinzufügen
main-remove-from-folder = Aus { $folder } entfernen
main-refresh-thumbnail = Vorschaubild aktualisieren
main-refreshing-thumbnail = Vorschaubild wird aktualisiert…
main-organize = Ordnen
main-organize-again = Erneut ordnen
main-organizing = Wird geordnet…
main-move-to-trash = In den Papierkorb legen
main-restore = Wiederherstellen
main-delete-now = Jetzt löschen
main-open-link = Link öffnen
main-card-failed = Konnte nicht geordnet werden
main-card-not-organized = Noch nicht geordnet
main-empty-no-events-match = Keine passenden Events
main-empty-no-events-match-body = Probier es mit weniger Wörtern oder einem Datumsfilter wie starts>2026-12-01.
main-empty-nothing-matches = Keine Treffer
main-empty-nothing-matches-body = Probier es mit weniger Wörtern oder Filtern wie kind:movie, site:github.com oder #tag.
main-empty-trash = Der Papierkorb ist leer
main-empty-trash-body = Gelöschte Lesezeichen warten hier 30 Tage lang.
main-empty-caught-up = Alles erledigt
main-empty-caught-up-body = Alle Lesezeichen hier sind geordnet.
main-empty-no-pictures = Keine Bilder hier
main-empty-no-pictures-body = Direkte Links zu Bilddateien erscheinen hier.
main-empty-no-events = Keine Events hier
main-empty-no-events-body = Seiten zu Konzerten, Vorträgen und Meetups erscheinen hier nach Datum.
main-empty-nothing-yet = Hier ist noch nichts
main-empty-nothing-yet-synced-body = Speichere ein Lesezeichen im Browser, dann landet es kurz darauf hier.
main-empty-nothing-yet-unsynced-body = Füge mit ⌘N einen Link hinzu oder wähle im Arbeitsbereich-Menü einen Browser.
main-empty-nothing-yet-body = Lesezeichen erscheinen hier, sobald du sie hinzufügst.
main-up-next = Als Nächstes
main-up-next-count =
    { $count ->
        [one] · { $count } zum Lesen und Anschauen
       *[other] · { $count } zum Lesen und Anschauen
    }
main-best-matches = Beste Treffer
main-place-gallery = Galerie
main-place-events = Events
main-place-up-next = Als Nächstes
main-place-unsorted = Ohne Ordner
main-place-not-organized = Nicht geordnet
main-place-set-aside = Zurückgestellt
main-place-trash = Papierkorb
main-search-found =
    { $count ->
        [one] { $count } gefunden
       *[other] { $count } gefunden
    }
main-kind-all = Alle
main-kind-chip = { $kind } { $count }
main-trash-empty = Papierkorb entleeren
main-trash-delete-count =
    { $count ->
        [one] { $count } endgültig löschen
       *[other] { $count } endgültig löschen
    }
main-trash-about = Lesezeichen im Papierkorb werden nach 30 Tagen endgültig gelöscht.
main-trash-days-left =
    { $days ->
        [1] Im Papierkorb. Wird innerhalb eines Tages endgültig gelöscht.
       *[other] Im Papierkorb. Wird in { $days } Tagen endgültig gelöscht.
    }
main-add-link-title = Link zu { $workspace } hinzufügen
main-add-link-not-a-link = Das sieht nicht nach einer Webadresse aus.
main-add-link-add = Hinzufügen
main-image-file-name = Bild
main-viewer-position = { $position } von { $count }
main-viewer-open-original = Original öffnen
main-viewer-close = Schließen
main-viewer-image-copied = Bild kopiert
main-viewer-copy-failed = Bild konnte nicht kopiert werden
main-viewer-image-saved = Bild gespeichert
main-viewer-save-failed = Bild konnte nicht gespeichert werden
main-search-placeholder = Einfach lostippen zum Suchen
main-view-options-title = Darstellung
main-view-options-summaries = Zusammenfassungen auf Karten anzeigen
main-view-options-up-next = Leiste „Als Nächstes“ anzeigen
main-view-options-about = Karten sind nach Datum gruppiert. Die Suche sortiert nach bester Übereinstimmung.
main-duration-minutes = { $minutes } Min.
main-duration-hours = { $hours } Std.
main-duration-hours-minutes = { $hours } Std. { $minutes } Min.

## Detail

detail-details = Details
detail-removed-from-browser = Im Browser entfernt, hier behalten
detail-saved = Gespeichert am { $date }
detail-chosen-by = Markiert, was { $model } gewählt hat
detail-folders = Ordner
detail-no-folders = Noch keine Ordner. Lege einen in der Seitenleiste an.
detail-folder-undo = Rückgängig: herausnehmen
detail-folder-take-out = Herausnehmen
detail-folder-probability = { $percent } %
detail-add-to-another-folder = Zu weiterem Ordner hinzufügen
detail-set-aside = Zurückgestellt: passt nicht in diesen Arbeitsbereich
detail-keep = Behalten
detail-organizing-now = Wird gerade geordnet…
detail-organize-failed = Das Ordnen hat nicht geklappt.
detail-try-again = Erneut versuchen
detail-title-and-link-only = Bisher nur Titel und Link.
detail-read-without-ai = Nur aus der Seite gelesen, ohne KI.
detail-no-thumbnail = Kein Vorschaubild auf der Seite
detail-page-unreachable = Seite nicht erreichbar
detail-more = Mehr
detail-tags = Tags
detail-remove-tag = Tag entfernen
detail-put-back = Wieder hinzufügen
detail-add-tag = Tag hinzufügen
detail-add-a-tag = Tag hinzufügen
detail-in-tag-list = In Tag-Liste
detail-in-tag-list-full = In Tag-Liste (Liste ist voll)
detail-edit-tag-list = Tag-Liste bearbeiten…

## Events

events-title = Events
events-view-calendar = Kalender
events-view-list = Liste
events-previous-month = Vorheriger Monat
events-next-month = Nächster Monat
events-today = Heute
events-upcoming = Bevorstehend
events-happening-now = Läuft gerade
events-date-not-set = Ohne Datum
events-past = Vergangen
events-nothing-on-day = Nichts an diesem Tag.
events-nothing-coming-up = Keine kommenden Events.
events-nothing-coming-up-panel = Keine kommenden Events. Gespeicherte Events erscheinen hier, bis sie enden.
events-more = +{ $count } weitere

## Dates

date-today = Heute
date-yesterday = Gestern
date-tomorrow = Morgen
date-this-week = Diese Woche
date-last-week = Letzte Woche
date-all-day = Ganztägig

## Kinds

kind-article = Artikel
kind-article-plural = Artikel
kind-video = Video
kind-video-plural = Videos
kind-movie = Film
kind-movie-plural = Filme
kind-recipe = Rezept
kind-recipe-plural = Rezepte
kind-product = Produkt
kind-product-plural = Produkte
kind-repository = Repository
kind-repository-plural = Code
kind-book = Buch
kind-book-plural = Bücher
kind-image = Bild
kind-image-plural = Bilder
kind-app = App
kind-app-plural = Apps
kind-podcast = Podcast
kind-podcast-plural = Podcasts
kind-place = Ort
kind-place-plural = Orte
kind-event = Event
kind-event-plural = Events
kind-other = Sonstiges
kind-other-plural = Sonstiges

## Fields

field-address = Adresse
field-attendance = Teilnahme
field-author = Autor
field-brand = Marke
field-cast = Besetzung
field-category = Kategorie
field-channel = Kanal
field-city = Stadt
field-colors = Farben
field-country = Land
field-cuisine = Küche
field-currency = Währung
field-diet = Ernährung
field-director = Regie
field-duration = Dauer
field-ends = Ende
field-format = Format
field-genre = Genre
field-host = Moderation
field-language = Sprache
field-license = Lizenz
field-maker = Entwickler
field-medium = Medium
field-name = Name
field-organizer = Veranstalter
field-owner = Inhaber
field-pages = Seiten
field-platform = Plattform
field-price = Preis
field-pricing = Preismodell
field-published = Veröffentlicht
field-publisher = Verlag
field-rating = Bewertung
field-reading-minutes = Lesezeit
field-resolution = Auflösung
field-runtime = Laufzeit
field-servings = Portionen
field-show = Show
field-stars = Sterne
field-starts = Beginn
field-total-time = Gesamtzeit
field-venue = Veranstaltungsort
field-visible-text = Sichtbarer Text
field-year = Jahr

## Inks

ink-ochre = Ocker
ink-olive = Oliv
ink-moss = Moos
ink-pine = Tanne
ink-teal = Petrol
ink-sky = Himmel
ink-cobalt = Kobalt
ink-violet = Violett
ink-plum = Pflaume
ink-rose = Rosé
ink-cocoa = Kakao
ink-slate = Schiefer

## Errors

error-cant-start = { $name } kann nicht gestartet werden
error-quit = Beenden
error-signed-out = Du wurdest abgemeldet. Melde dich wieder an, um weiter zu synchronisieren.
error-sync-unanswered = Der Server hat die letzten Änderungen nicht angenommen.
error-unlink-library = Verknüpfung der Bibliothek konnte nicht aufgehoben werden: { $error }
error-mcp-load = KI-Zugriff konnte nicht geladen werden: { $problem }
error-mcp-save = KI-Zugriff konnte nicht gespeichert werden: { $problem }
error-mcp-needs-pro = KI-Zugriff erfordert { -knoa } Pro.
