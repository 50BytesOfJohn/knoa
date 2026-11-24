## Terms
## One word per concept, used everywhere in this language.

-knoa = Knoa
-workspace = przestrzeń
-bookmark = zakładka
-folder = folder
-tag = tag
-summary = podsumowanie
-organize = porządkować

## Menus

menu-about = O aplikacji { $name }
menu-check-for-updates = Sprawdź aktualizacje…
menu-settings = Ustawienia…
menu-services = Usługi
menu-hide = Ukryj { $name }
menu-hide-others = Ukryj pozostałe
menu-show-all = Pokaż wszystkie
menu-quit = Zakończ { $name }
menu-help = Pomoc
menu-whats-new = Co nowego w { $name } { $version }
menu-report-issue = Zgłoś problem…

## Onboarding

onboarding-skip-setup = Pomiń konfigurację →
onboarding-back = Wróć
onboarding-next = Dalej
onboarding-hello-title = Cześć, jestem { -knoa }.
onboarding-hello-lead = Zapisujesz zakładki jak wiewiórka robiąca zapasy na zimę. Ja pomagam je potem znaleźć. Czytam zakładki z Twojej przeglądarki, sprawdzam, co jest czym, i utrzymuję w nich porządek.
onboarding-hello-questions = Trzy szybkie pytania. Albo żadnego, jeśli Ci się spieszy.
onboarding-hello-start = Zaczynamy
onboarding-hello-defaults = Użyj domyślnych
onboarding-hello-have-account = Masz już konto { -knoa }?
onboarding-hello-sign-in = Zaloguj się
onboarding-browser-title = Gdzie mieszkają Twoje zakładki?
onboarding-browser-lead = Zapisuj zakładki tak jak dotychczas. Nowe zauważę w kilka sekund. Tylko czytam — nigdy nie ruszam zakładek w Twojej przeglądarce.
onboarding-browser-count =
    { $count ->
        [one] { $count } zakładka
        [few] { $count } zakładki
        [many] { $count } zakładek
       *[other] { $count } zakładek
    }
onboarding-browser-none-found = Nie widzę tu żadnej przeglądarki opartej na Chromium (Helium, Chrome, Brave, Edge, Vivaldi). Nic nie szkodzi — możesz ją wybrać później w Ustawieniach.
onboarding-browser-bring-in = Zaimportuj { $count }
onboarding-ai-title = Mam zająć się porządkowaniem?
onboarding-ai-lead = Z kluczem OpenRouter przeczytam każdą nową stronę i nadam jej rodzaj, podsumowanie, tagi i folder. To Twój klucz i Twoje konto: około 17 centów za tysiąc zakładek.
onboarding-ai-key-placeholder = Wklej swój klucz: sk-or-v1-…
onboarding-ai-check-key = Sprawdź klucz
onboarding-ai-checking = Pukam do drzwi OpenRouter…
onboarding-ai-key-works = Ten klucz działa.
onboarding-ai-key-missing = Najpierw wklej klucz albo pomiń ten krok.
onboarding-ai-key-unknown = OpenRouter nie rozpoznaje tego klucza. Sprawdzisz go jeszcze raz?
onboarding-ai-key-check-failed = Nie udało się sprawdzić klucza: { $error }
onboarding-ai-get-key = Uzyskaj klucz ↗
onboarding-ai-without-key = Bez niego nadal pobieram obrazy i opisy.
onboarding-ai-skip = Pomiń, bez AI
onboarding-library-title = Ostatnia rzecz: jak nazwać tę bibliotekę?
onboarding-library-default-name = Prywatne
onboarding-library-house-rules = Jakieś zasady porządkowania?
onboarding-library-instructions-placeholder = Opcjonalnie. Napisz tak, jak znajomemu.
onboarding-library-example-apart = Oddzielaj sprawy służbowe od prywatnych.
onboarding-library-example-recipes = Taguj przepisy według kuchni.
onboarding-library-example-rust = Wszystko o Ruście trafia do folderu Rust.
onboarding-library-example-short = Krótkie podsumowania, bez lania wody.
onboarding-library-add-example = + { $example }
onboarding-library-finish = Gotowe
onboarding-done-title = Wszystko gotowe.
onboarding-done-no-browser = Na razie bez przeglądarki, więc na początek biblioteka jest pusta. Wybierz ją w Ustawieniach, kiedy tylko zechcesz.
onboarding-done-bringing-in = Importowanie Twoich zakładek…
onboarding-done-count =
    { $count ->
        [one] { $count } zakładka jest już w bibliotece — możesz szukać od razu.
        [few] { $count } zakładki są już w bibliotece — możesz szukać od razu.
        [many] { $count } zakładek jest już w bibliotece — możesz szukać od razu.
       *[other] { $count } zakładek jest już w bibliotece — możesz szukać od razu.
    }
onboarding-done-organize-existing =
    { $count ->
        [one] Nowe zakładki porządkują się same. Mam przejrzeć też { $count } zakładkę, którą już masz? Około { $cost }.
        [few] Nowe zakładki porządkują się same. Mam przejrzeć też { $count } zakładki, które już masz? Około { $cost }.
        [many] Nowe zakładki porządkują się same. Mam przejrzeć też { $count } zakładek, które już masz? Około { $cost }.
       *[other] Nowe zakładki porządkują się same. Mam przejrzeć też { $count } zakładek, które już masz? Około { $cost }.
    }
onboarding-done-organize = Tak, uporządkuj
onboarding-done-open-library = Otwórz moją bibliotekę

## Settings

settings-title = Ustawienia
settings-workspaces = Przestrzenie
settings-new-workspace = Nowa przestrzeń
settings-nav-general = Ogólne
settings-nav-appearance = Wygląd
settings-nav-ai = AI
settings-nav-account = Konto
settings-nav-organize = Porządkowanie
settings-nav-mcp = MCP
settings-general-title = Ogólne
settings-general-about = Gdzie { -knoa } trzyma Twoją bibliotekę, jak skonfigurować aplikację i jak działają aktualizacje.
settings-general-library = Biblioteka
settings-general-location = Położenie biblioteki
settings-general-show-in-finder = Pokaż w Finderze
settings-general-setup = Konfiguracja
settings-general-run-setup-again = Uruchom konfigurację ponownie
settings-general-run-setup-about = Wybierz przeglądarki, AI i instrukcje od nowa. Twoje zakładki zostaną.
settings-general-run-setup = Uruchom konfigurację
settings-general-language = Język
settings-general-language-label = Język aplikacji i AI
settings-general-language-about = Menu, ustawienia i daty oraz podsumowania i tagi pisane przez AI.
settings-general-language-note = Już napisane podsumowania i tagi pozostają w poprzednim języku. Ponowne uporządkowanie biblioteki napisze je od nowa.
settings-general-language-organize = Uporządkuj ponownie…
settings-general-language-organize-confirm = Uporządkuj wszystkie
settings-general-language-waiting = Uporządkujesz ponownie, gdy twoje konto będzie już miało nowy język.
settings-updates = Aktualizacje
settings-updates-version = { -knoa } { $version }
settings-updates-self-built = Ta kompilacja nie aktualizuje się sama. Wydania z GitHuba już tak.
settings-updates-checked-today = Ostatnio sprawdzono dzisiaj o { $time }
settings-updates-checked-yesterday = Ostatnio sprawdzono wczoraj o { $time }
settings-updates-checked-on = Ostatnio sprawdzono { $date }
settings-updates-not-checked = Jeszcze nie sprawdzano
settings-updates-check-now = Sprawdź teraz
settings-updates-check-automatically = Sprawdzaj aktualizacje automatycznie
settings-updates-check-automatically-about = Raz dziennie, w tle.
settings-updates-install-automatically = Instaluj aktualizacje automatycznie
settings-updates-install-automatically-about = Pobiera nowe wersje i instaluje je przy zamykaniu { -knoa }.
settings-appearance-title = Wygląd
settings-appearance-about = Jak wygląda { -knoa }. Zmiany widać od razu.
settings-appearance-theme = Motyw
settings-appearance-light = Jasny
settings-appearance-dark = Ciemny
settings-appearance-library = Biblioteka
settings-appearance-summaries = Pokazuj podsumowania na kartach
settings-appearance-summaries-about = Do trzech wierszy pod każdym tytułem.
settings-appearance-up-next = Pokazuj pasek Na później
settings-appearance-up-next-about = Rząd rzeczy do przeczytania lub obejrzenia na górze widoku Wszystko.
settings-appearance-events = Pokazuj wydarzenia jako
settings-appearance-events-about = Kalendarz miesięczny z nadchodzącymi wydarzeniami albo jedna lista. Tylko na tym Macu.
settings-appearance-library-order = Karty są grupowane według daty. Wyszukiwanie sortuje według trafności.
settings-reorganize-title = Uporządkuj ponownie
settings-reorganize-folders-confirm = Ułóż wszystkie
settings-reorganize-everything-confirm = Uporządkuj wszystkie
settings-organize-title = Porządkowanie
settings-organize-about = Niech AI uporządkuje zakładki, do których jeszcze nie dotarło, albo przejrzy ponownie te już uporządkowane.
settings-organize-scope = Zakładki w
settings-organize-scope-all = Wszystkie przestrzenie
settings-organize-new = Jeszcze nieuporządkowane
settings-organize-counting = Liczenie zakładek…
settings-organize-none = Tu wszystko jest uporządkowane.
settings-organize-waiting =
    { $count ->
        [one] { $count } zakładka nie jest jeszcze uporządkowana
        [few] { $count } zakładki nie są jeszcze uporządkowane
        [many] { $count } zakładek nie jest jeszcze uporządkowanych
       *[other] { $count } zakładek nie jest jeszcze uporządkowanych
    }
settings-organize-ai-off = Porządkowanie wymaga AI
settings-organize-ai-off-about = Podłącz własny klucz albo wybierz Pro na stronie AI.
settings-organize-open-ai = Skonfiguruj AI
settings-reorganize-folders-label = Tylko foldery
settings-reorganize-folders-about = Układa uporządkowane zakładki w obecnych folderach. Strony nie są czytane ponownie, więc to szybkie i tanie.
settings-reorganize-folders-start = Ułóż ponownie…
settings-reorganize-everything-label = Wszystko
settings-reorganize-everything-about = Czyta każdą stronę ponownie i od nowa tworzy podsumowanie, pola, tagi i foldery według obecnych instrukcji i języka.
settings-reorganize-everything-start = Uporządkuj ponownie…
settings-reorganize-kept = Foldery ustawione lub usunięte przez Ciebie zostają bez zmian.
settings-reorganize-queued =
    { $count ->
        [one] Dodano do kolejki { $count } zakładkę.
        [few] Dodano do kolejki { $count } zakładki.
        [many] Dodano do kolejki { $count } zakładek.
       *[other] Dodano do kolejki { $count } zakładek.
    }
settings-account-synced-today = Ostatnio zsynchronizowano dzisiaj o { $time }
settings-account-synced-yesterday = Ostatnio zsynchronizowano wczoraj o { $time }
settings-account-synced-on = Ostatnio zsynchronizowano { $date }
settings-account-not-synced = Jeszcze nie zsynchronizowano
settings-account-start-trial = Rozpocznij bezpłatny okres próbny
settings-account-upgrade = Zmień plan
settings-account-manage-subscription = Zarządzaj subskrypcją
settings-account-usage = Wykorzystanie
settings-account-usage-device = { $used } z { $limit } zakładek
settings-account-usage-cloud = { $used } z { $limit } zakładek, { $organized } z { $allowance } uporządkowanych w tym miesiącu
settings-account-plan-free = Plan darmowy.
settings-account-plan-trial = Okres próbny Pro.
settings-account-plan-pro = Plan Pro.
settings-account-plan-ends = Kończy się { $date }.
settings-account-plan-trial-ends = Okres próbny kończy się { $date }.
settings-account-plan-renews = Odnawia się { $date }.
settings-account-organized-on-device = Porządkowanie używa Twojego klucza AI na tym Macu.
settings-account-organized-in-cloud = { -knoa } porządkuje Twoje zakładki w chmurze.
settings-account-title = Konto
settings-account-about = Synchronizuj bibliotekę z kontem { -knoa }.
settings-account-unavailable = Synchronizacja w chmurze nie jest dostępna w tej kompilacji.
settings-account-couldnt-start = Nie udało się uruchomić: { $reason }
settings-account-cloud-mode = Tryb chmurowy
settings-account-what-it-is = Tryb chmurowy synchronizuje bibliotekę między Twoimi urządzeniami oraz z { -knoa } w przeglądarce. W planie Pro { -knoa } porządkuje też zakładki w chmurze, bez Twojego klucza.
settings-account-privacy = W trybie chmurowym Twoja biblioteka jest przechowywana na serwerach { -knoa }, które czytają Twoje strony, żeby je uporządkować. Dane nie są szyfrowane end-to-end. Po wylogowaniu wszystko zostaje na tym Macu.
settings-account-finish-signing-in = Dokończ logowanie w przeglądarce…
settings-account-finish-signing-in-about = Strona logowania { -knoa } otworzyła się w Twojej przeglądarce.
settings-account-cancel = Anuluj
settings-account-knoa-account = Konto { -knoa }
settings-account-sign-in-about = Zaloguj się lub utwórz konto w przeglądarce.
settings-account-sign-in = Zaloguj się
settings-account-other-account = Ta biblioteka jest zsynchronizowana z innym kontem { -knoa }. Nic się nie zsynchronizuje, dopóki nie wybierzesz.
settings-account-sync-with = Zamiast tego synchronizuj z { $email }
settings-account-sync-with-this-account = Zamiast tego synchronizuj z tym kontem
settings-account-sync-with-about = Biblioteka zostanie połączona z tym kontem. Drugie konto zachowa swoją kopię.
settings-account-use-this-account = Użyj tego konta
settings-account-this-account = To konto
settings-account-asking = Zalogowano. Pobieranie informacji o koncie z serwera…
settings-account-sync = Synchronizacja
settings-account-connecting = Łączenie…
settings-account-up-to-date = Aktualne
settings-account-syncing =
    { $count ->
        [one] Synchronizowanie { $count } zmiany…
        [few] Synchronizowanie { $count } zmian…
        [many] Synchronizowanie { $count } zmian…
       *[other] Synchronizowanie { $count } zmian…
    }
settings-account-offline = Offline. Zmiany zsynchronizują się, gdy { -knoa } połączy się z serwerem.
settings-account-sync-stopped = Synchronizacja zatrzymana: { $error }
settings-account-notice-plan-limit =
    { $unsent ->
        [one] Twój plan mieści { $limit } zakładek, więc { $unsent } nowa zakładka została tylko na tym Macu.
        [few] Twój plan mieści { $limit } zakładek, więc { $unsent } nowe zakładki zostały tylko na tym Macu.
        [many] Twój plan mieści { $limit } zakładek, więc { $unsent } nowych zakładek zostało tylko na tym Macu.
       *[other] Twój plan mieści { $limit } zakładek, więc { $unsent } nowych zakładek zostało tylko na tym Macu.
    }
settings-account-notice-rejected = Część zmian nie mogła się zsynchronizować i została tylko na tym Macu.
settings-account-notice-too-large = Część zmian była zbyt duża do synchronizacji i została tylko na tym Macu.
settings-account-notice-allowance-used-up = Miesięczny limit porządkowania został wyczerpany. { -knoa } zachowa Twoje zlecenia porządkowania i wznowi je automatycznie, gdy limit się odnowi.
settings-account-sign-out = Wyloguj się
settings-account-sign-out-about = Twoja biblioteka zostanie na tym Macu. Zaloguj się ponownie, żeby wrócić do synchronizacji.
settings-ai-access-about = Pozwól aplikacjom AI, takim jak Claude i ChatGPT, czytać Twoje zakładki i nimi zarządzać.
settings-ai-access-copy = Kopiuj
settings-ai-access-url = Adres URL łącznika
settings-ai-access-default = Domyślny dostęp
settings-ai-access-same-as-default = Tak jak domyślny
settings-ai-access-off = Wyłączony
settings-ai-access-read = Tylko odczyt
settings-ai-access-full = Pełny dostęp
settings-ai-access-not-linked = Dostępne po połączeniu tej przestrzeni z chmurą.
settings-ai-access-saving = Zapisywanie…
settings-ai-access-loading = Wczytywanie dostępu AI…
settings-ai-access-save-again = Zapisz dostęp AI ponownie
settings-ai-access-load-again = Wczytaj dostęp AI ponownie
settings-ai-access-retry = Ponów
settings-ai-access-needs-pro = Dostęp AI wymaga { -knoa } Pro
settings-mcp-title = MCP
settings-mcp-sign-in = Zaloguj się, aby połączyć aplikacje AI
settings-mcp-sign-in-about = Aplikacje AI łączą się z Twoją biblioteką przez konto { -knoa }.
settings-mcp-open-account = Otwórz Konto
settings-mcp-pro-about = Z Pro aplikacje AI, takie jak Claude i ChatGPT, mogą przeszukiwać, czytać i porządkować Twoją bibliotekę.
settings-mcp-copied = Skopiowano
settings-mcp-connect = Połączenie
settings-mcp-how = Jak się połączyć
settings-mcp-how-copy = Skopiuj adres URL łącznika.
settings-mcp-how-add = W Claude, ChatGPT lub innej aplikacji AI dodaj go jako własny łącznik.
settings-mcp-how-sign-in = Zaloguj się do { -knoa }, gdy aplikacja o to poprosi.
settings-mcp-how-about = Połączona aplikacja AI może czytać Twoje zakładki, w tym tekst zapisany z ich stron, w każdej przestrzeni, do której ma dostęp. Zmiany wprowadzone tutaj zadziałają od następnego zapytania.
settings-mcp-access = Dostęp
settings-mcp-default-off = Aplikacje AI nie mają dostępu do przestrzeni z ustawieniem domyślnym.
settings-mcp-default-read = Aplikacje AI mogą przeszukiwać i czytać zakładki w przestrzeniach z ustawieniem domyślnym.
settings-mcp-default-full = Aplikacje AI mogą też dodawać, edytować i przenosić zakładki do Kosza oraz zarządzać folderami i przestrzeniami.
settings-ai-title = AI
settings-ai-about = { -knoa } działa też bez AI. Z nim nowe zakładki dostają rodzaj, podsumowanie, tagi i foldery.
settings-ai-provider = Dostawca
settings-ai-cloud-note = W planie Pro { -knoa } porządkuje Twoje zakładki w chmurze. Połączony wcześniej klucz zostaje na tym Macu na wypadek wylogowania.
settings-ai-key = Klucz OpenRouter
settings-ai-key-about = Twój własny klucz — zapytania idą z tego Maca prosto do OpenRouter.
settings-ai-connect = Połącz
settings-ai-checking = Sprawdzanie klucza…
settings-ai-key-connected = Połączono. Od teraz nowe zakładki będą porządkowane.
settings-ai-key-failed = To nie zadziałało: { $error }.
settings-ai-no-key = Nie masz jeszcze klucza?
settings-ai-no-key-about = Kilka dolarów środków wystarcza na tysiące zakładek.
settings-ai-get-key = Uzyskaj klucz
settings-ai-connected = Połączono
settings-ai-disconnect = Rozłącz
settings-ai-openrouter-says = OpenRouter odpowiada: { $problem }. Sprawdź swój klucz lub środki.
settings-ai-models = Modele
settings-ai-included-in-pro = Wliczone w plan Pro, w ramach miesięcznego limitu.
settings-ai-cost = Około { $cost } za wszystkie.
settings-ai-organize = Uporządkuj
settings-ai-pro-title = Zostaw to { -knoa } w planie Pro
settings-ai-pro-pitch = { -knoa } porządkuje Twoje zakładki w chmurze. Bez konfiguracji klucza, bez rozliczania za każde zapytanie, a Twoja biblioteka synchronizuje się między urządzeniami.
settings-ai-pro-get = Wybierz Pro
settings-ai-pro-upgrade = Przejdź na Pro
settings-ai-pro-have = Mam już Pro
settings-ai-usage = Wykorzystanie
settings-ai-usage-none = Brak zapytań AI.
settings-ai-usage-recent =
    { $days ->
        [one] Ostatni { $days } dzień
        [few] Ostatnie { $days } dni
        [many] Ostatnie { $days } dni
       *[other] Ostatnie { $days } dni
    }
settings-ai-usage-all-time = Od początku
settings-ai-usage-by-model =
    { $days ->
        [one] Według modelu, ostatni { $days } dzień
        [few] Według modelu, ostatnie { $days } dni
        [many] Według modelu, ostatnie { $days } dni
       *[other] Według modelu, ostatnie { $days } dni
    }
settings-ai-usage-records = Historia w { $provider }
settings-ai-usage-records-about = Każde zapytanie wraz z dokładną pobraną kwotą.
settings-ai-usage-open = Otwórz
settings-ai-usage-requests =
    { $count ->
        [one] { $count } zapytanie
        [few] { $count } zapytania
        [many] { $count } zapytań
       *[other] { $count } zapytań
    }
settings-ai-usage-tokens = { $input } tokenów wej., { $output } wyj.
settings-ai-usage-unmetered =
    { $count ->
        [one] { $count } bez liczby tokenów
        [few] { $count } bez liczby tokenów
        [many] { $count } bez liczby tokenów
       *[other] { $count } bez liczby tokenów
    }
settings-ai-usage-unpriced =
    { $count ->
        [one] { $count } bez ceny
        [few] { $count } bez ceny
        [many] { $count } bez ceny
       *[other] { $count } bez ceny
    }
settings-workspace-new = Nowa przestrzeń
settings-workspace-new-about = Osobna biblioteka, np. Praca albo Prywatne, z własnymi przeglądarkami i folderami.
settings-workspace-shown-now = Teraz otwarta
settings-workspace-show-it = Pokaż
settings-workspace-name-and-color = Nazwa i kolor
settings-workspace-name = Nazwa
settings-workspace-name-too-long = Nazwa może mieć maksymalnie { $max } znaków.
settings-workspace-color = Kolor
settings-workspace-no-browser = Nie znaleziono przeglądarki opartej na Chromium. { -knoa } obsługuje Helium, Chrome, Brave, Edge, Vivaldi i Chromium.
settings-workspace-browsers-note = Gdy żadna nie jest włączona, linki dodajesz tylko ręcznie (⌘N). Jedna przeglądarka może zasilać kilka przestrzeni.
settings-workspace-open-in = Otwieraj zakładki w
settings-workspace-browsers = Przeglądarki
settings-workspace-open-in-about = Gdzie otwierają się linki z tej przestrzeni.
settings-workspace-organize-again-about = Po zmianie instrukcji lub tagów ułóż albo uporządkuj ponownie zakładki tej przestrzeni.
settings-workspace-open-organize = Uporządkuj…
settings-workspace-default-browser = Domyślna przeglądarka
settings-workspace-by-hand = Tylko linki dodane ręcznie.
settings-workspace-synced-from-one = Synchronizacja z { $browser }.
settings-workspace-synced-from-many = Synchronizacja z { $browsers } i { $last }.
settings-workspace-organizing = Porządkowanie
settings-workspace-instructions = Instrukcje
settings-workspace-instructions-about = Jak AI ma układać, tagować i podsumowywać wszystko, co tu trafia.
settings-workspace-instructions-too-long = Instrukcje mogą mieć maksymalnie { $max } znaków.
settings-workspace-screening = Zachowuj tylko pasujące zakładki
settings-workspace-screening-about = Wył.: wszystko z przeglądarki trafia tutaj. Wł.: AI odkłada to, co nie pasuje, ale zawsze możesz to zachować.
settings-workspace-guideline = Co tu pasuje
settings-workspace-guideline-too-long = Wskazówka może mieć maksymalnie { $max } znaków.
settings-workspace-name-placeholder = Praca, Prywatne…
settings-workspace-instructions-placeholder = np. Taguj przepisy według kuchni. Krótkie podsumowania, bez lania wody.
settings-workspace-guideline-placeholder = np. Tylko rzeczy do pracy: programowanie, design i nasi klienci.
settings-workspace-create = Utwórz przestrzeń
settings-workspace-delete = Usuwanie
settings-workspace-delete-label = Usuń tę przestrzeń
settings-workspace-delete-about = Jej zakładki i foldery znikną razem z nią. Przeglądarki zachowają swoje zakładki.
settings-workspace-delete-button = Usuń…
settings-workspace-delete-confirm = Usuń na zawsze
settings-tags-title = Lista tagów
settings-tags-label = Tagi, których ma używać AI
settings-tags-about = Gdy któryś z nich pasuje do zakładki, AI używa go zamiast własnego słowa. Krótka wskazówka podpowiada, kiedy go użyć. Możesz też kliknąć prawym przyciskiem tag na zakładce, żeby dodać go do listy.
settings-tags-add-placeholder = Dodaj tag
settings-tags-hint-placeholder = Kiedy używać (opcjonalnie)
settings-tags-remove = Usuń z listy
settings-tags-only-listed = Używaj tylko tagów z tej listy
settings-tags-only-listed-about = Wył.: AI dodaje własne tagi tam, gdzie lista nie wystarcza. Wł.: zakładki dostają tylko tagi z listy lub żadne, jeśli nic nie pasuje.
settings-tags-only-listed-about-empty = Najpierw dodaj tag. Wył.: AI dodaje własne tagi tam, gdzie lista nie wystarcza. Wł.: zakładki dostają tylko tagi z listy lub żadne, jeśli nic nie pasuje.
settings-tags-note = Zakładki porządkowane od teraz dostaną te tagi. Aby dodać je do już uporządkowanych, uporządkuj je ponownie.
settings-tags-count = { $count } z { $max }

## Sidebar

sidebar-status-ai-needs-attention = AI wymaga uwagi
sidebar-status-fix = Napraw
sidebar-sync-stopped = Synchronizacja zatrzymana
sidebar-retry = Ponów
sidebar-sync-paused = Synchronizacja wstrzymana
sidebar-status-changes-not-synced = Nie zsynchronizowano zmian
sidebar-status-view = Zobacz
sidebar-status-plan-limit = Osiągnięto limit planu
sidebar-status-upgrade = Zmień plan
sidebar-status-organizing = Porządkowanie { $current } z { $total }…
sidebar-status-ai-off = AI jest wyłączone
sidebar-status-turn-on = Włącz
sidebar-status-offline = Offline. Zmiany zsynchronizują się później.
sidebar-syncing = Synchronizowanie…
sidebar-sync-offline =
    { $count ->
        [0] Offline
        [one] Offline · { $count } zmiana czeka
        [few] Offline · { $count } zmiany czekają
        [many] Offline · { $count } zmian czeka
       *[other] Offline · { $count } zmian czeka
    }
sidebar-synced-just-now = Zsynchronizowano przed chwilą
sidebar-synced-minutes-ago =
    { $minutes ->
        [one] Zsynchronizowano { $minutes } min temu
        [few] Zsynchronizowano { $minutes } min temu
        [many] Zsynchronizowano { $minutes } min temu
       *[other] Zsynchronizowano { $minutes } min temu
    }
sidebar-synced-hours-ago =
    { $hours ->
        [one] Zsynchronizowano { $hours } godz. temu
        [few] Zsynchronizowano { $hours } godz. temu
        [many] Zsynchronizowano { $hours } godz. temu
       *[other] Zsynchronizowano { $hours } godz. temu
    }
sidebar-account-settings = Ustawienia konta…
sidebar-open-web = Otwórz { -knoa } w przeglądarce
sidebar-sign-out = Wyloguj się
sidebar-sign-in = Zaloguj się…
sidebar-finish-in-browser = Dokończ w przeglądarce
sidebar-cancel = Anuluj
sidebar-plan-pro = { -knoa } Pro
sidebar-plan-free = Plan darmowy
sidebar-signed-in = Zalogowano
sidebar-manage-plan = Zarządzaj planem…
sidebar-upgrade-to-pro = Przejdź na Pro
sidebar-this-mac = Ten Mac
sidebar-knoa-account = Konto { -knoa }
sidebar-place-everything = Wszystko
sidebar-place-gallery = Galeria
sidebar-place-events = Wydarzenia
sidebar-place-up-next = Na później
sidebar-place-unsorted = Bez folderu
sidebar-place-not-organized = Nieuporządkowane
sidebar-place-set-aside = Odłożone
sidebar-place-trash = Kosz
sidebar-settings = Ustawienia
sidebar-folders = Foldery
sidebar-new-folder = Nowy folder
sidebar-folder-new-subfolder = Nowy podfolder
sidebar-folder-edit = Edytuj folder…
sidebar-folder-delete = Usuń…
sidebar-folder-move-to = Przenieś do
sidebar-folder-top-level = Poziom główny
sidebar-add-link = Dodaj link
sidebar-workspaces-title = Przestrzenie
sidebar-workspaces-new = Nowa przestrzeń
sidebar-workspaces-by-hand = Linki dodane ręcznie
sidebar-workspaces-from = Z { $browsers }

## Main

main-blocked-browsers =
    { $count ->
        [one] Przeglądarka { $browsers } jest zainstalowana, ale macOS chroni jej zakładki. Przyznaj { -knoa } Pełny dostęp do dysku, a potem uruchom { -knoa } ponownie.
        [few] Przeglądarki { $browsers } są zainstalowane, ale macOS chroni ich zakładki. Przyznaj { -knoa } Pełny dostęp do dysku, a potem uruchom { -knoa } ponownie.
        [many] Przeglądarki { $browsers } są zainstalowane, ale macOS chroni ich zakładki. Przyznaj { -knoa } Pełny dostęp do dysku, a potem uruchom { -knoa } ponownie.
       *[other] Przeglądarki { $browsers } są zainstalowane, ale macOS chroni ich zakładki. Przyznaj { -knoa } Pełny dostęp do dysku, a potem uruchom { -knoa } ponownie.
    }
main-open-privacy-settings = Otwórz Ustawienia prywatności
main-cancel = Anuluj
main-delete = Usuń
main-save = Zapisz
main-folder-new-title = Nowy folder
main-folder-edit-title = Edytuj folder
main-folder-name-placeholder = Nazwa folderu
main-folder-description-placeholder = Co tu pasuje? AI używa tego opisu, żeby przydzielać zakładki.
main-folder-name-too-long = Nazwa może mieć maksymalnie { $max } znaków.
main-folder-description-too-long = Opis może mieć maksymalnie { $max } znaków.
main-folder-auto-assign = Pozwól AI dodawać zakładki
main-folder-auto-assign-about = Nowe pasujące zakładki trafią tu automatycznie.
main-glyph-read = Czytanie
main-glyph-article = Artykuły
main-glyph-code = Kod
main-glyph-write = Pisanie
main-glyph-work = Praca
main-glyph-tag = Okazje
main-glyph-bag = Zakupy
main-glyph-plane = Podróże
main-glyph-cup = Jedzenie
main-glyph-music = Muzyka
main-glyph-play = Wideo
main-glyph-photo = Zdjęcia
main-glyph-bulb = Pomysły
main-glyph-heart = Zdrowie
main-glyph-home = Dom
main-glyph-star = Ulubione
main-glyph-clock = Później
main-glyph-flask = Nauka
main-glyph-globe = Sieć
main-glyph-person = Ludzie
main-glyph-box = Archiwum
main-glyph-inbox = Skrzynka
main-glyph-leaf = Natura
main-glyph-bolt = Szybkie
main-glyph-lock = Prywatne
main-glyph-pin = Miejsca
main-glyph-chat = Wątki
main-glyph-game = Gry
main-folder-icon-none = Folder
main-folder-icon-emoji = Emoji
main-folder-parent = Nadrzędny
main-folder-parent-none = Brak
main-folder-emoji-placeholder = Wpisz lub wklej
main-folder-any-emoji = Dowolne emoji
main-folder-custom-color = Własny
main-delete-folder-title = Usunąć „{ $name }”?
main-delete-folder-detail =
    { $count ->
        [0] Jego zakładki pozostaną w Twojej bibliotece.
        [one] Jego podfolder również zostanie usunięty. Zakładki pozostaną w Twojej bibliotece.
        [few] Jego { $count } podfoldery również zostaną usunięte. Zakładki pozostaną w Twojej bibliotece.
        [many] Jego { $count } podfolderów również zostanie usuniętych. Zakładki pozostaną w Twojej bibliotece.
       *[other] Jego { $count } podfolderów również zostanie usuniętych. Zakładki pozostaną w Twojej bibliotece.
    }
main-open = Otwórz
main-open-in-background = Otwórz w tle
main-copy-link = Kopiuj link
main-copied = Skopiowano
main-show-details = Pokaż szczegóły
main-view-picture = Pokaż obraz
main-copy-image = Kopiuj obraz
main-save-image = Zapisz obraz…
main-add-to-up-next = Dodaj do Na później
main-remove-from-up-next = Usuń z Na później
main-add-to-folder = Dodaj do folderu
main-remove-from-folder = Usuń z „{ $folder }”
main-refresh-thumbnail = Odśwież miniaturę
main-refreshing-thumbnail = Odświeżanie miniatury…
main-organize = Uporządkuj
main-organize-again = Uporządkuj ponownie
main-organizing = Porządkowanie…
main-move-to-trash = Przenieś do Kosza
main-restore = Przywróć
main-delete-now = Usuń teraz
main-open-link = Otwórz link
main-card-failed = Nie udało się uporządkować
main-card-not-organized = Jeszcze nieuporządkowana
main-empty-no-events-match = Brak pasujących wydarzeń
main-empty-no-events-match-body = Spróbuj użyć mniej słów albo filtra daty, np. starts>2026-12-01.
main-empty-nothing-matches = Nic nie pasuje
main-empty-nothing-matches-body = Spróbuj użyć mniej słów albo filtrów, np. kind:film, site:github.com lub #tag.
main-empty-trash = Kosz jest pusty
main-empty-trash-body = Usunięte zakładki czekają tutaj przez 30 dni.
main-empty-caught-up = Wszystko na bieżąco
main-empty-caught-up-body = Każda zakładka tutaj jest już uporządkowana.
main-empty-no-pictures = Brak obrazów
main-empty-no-pictures-body = Bezpośrednie linki do plików graficznych pojawiają się tutaj.
main-empty-no-events = Brak wydarzeń
main-empty-no-events-body = Strony koncertów, prelekcji i spotkań pojawiają się tutaj według daty.
main-empty-nothing-yet = Na razie tu pusto
main-empty-nothing-yet-synced-body = Zapisz zakładkę w przeglądarce, a po chwili pojawi się tutaj.
main-empty-nothing-yet-unsynced-body = Dodaj link skrótem ⌘N albo wybierz przeglądarkę dla tej przestrzeni w menu przestrzeni.
main-empty-nothing-yet-body = Zakładki pojawiają się tutaj, gdy je dodasz.
main-up-next = Na później
main-up-next-count =
    { $count ->
        [one] · { $count } do przeczytania lub obejrzenia
        [few] · { $count } do przeczytania lub obejrzenia
        [many] · { $count } do przeczytania lub obejrzenia
       *[other] · { $count } do przeczytania lub obejrzenia
    }
main-best-matches = Najlepsze dopasowania
main-place-gallery = Galeria
main-place-events = Wydarzenia
main-place-up-next = Na później
main-place-unsorted = Bez folderu
main-place-not-organized = Nieuporządkowane
main-place-set-aside = Odłożone
main-place-trash = Kosz
main-search-found =
    { $count ->
        [one] { $count } wynik
        [few] { $count } wyniki
        [many] { $count } wyników
       *[other] { $count } wyników
    }
main-trash-empty = Opróżnij Kosz
main-trash-delete-count =
    { $count ->
        [one] Usuń { $count } na zawsze
        [few] Usuń { $count } na zawsze
        [many] Usuń { $count } na zawsze
       *[other] Usuń { $count } na zawsze
    }
main-trash-about = Zakładki w Koszu są usuwane na zawsze po 30 dniach.
main-trash-days-left =
    { $days ->
        [1] W Koszu. Zostanie usunięta na zawsze w ciągu dnia.
        [one] W Koszu. Zostanie usunięta na zawsze w ciągu dnia.
        [few] W Koszu. Zostanie usunięta na zawsze za { $days } dni.
        [many] W Koszu. Zostanie usunięta na zawsze za { $days } dni.
       *[other] W Koszu. Zostanie usunięta na zawsze za { $days } dni.
    }
main-add-link-title = Dodaj link do „{ $workspace }”
main-add-link-not-a-link = To nie wygląda na adres internetowy.
main-add-link-add = Dodaj
main-image-file-name = Obraz
main-viewer-position = { $position } z { $count }
main-viewer-open-original = Otwórz oryginał
main-viewer-close = Zamknij
main-viewer-image-copied = Skopiowano obraz
main-viewer-copy-failed = Nie udało się skopiować obrazu
main-viewer-image-saved = Zapisano obraz
main-viewer-save-failed = Nie udało się zapisać obrazu
main-search-placeholder = Zacznij pisać, żeby wyszukać
main-token-scope = w: { $place }
main-token-text = „{ $text }”
main-token-field = { $field }: { $value }
main-token-field-more = { $field } > { $value }
main-token-field-less = { $field } < { $value }
main-token-clear-all = Wyczyść wszystko
main-suggest-search-for = Szukaj „{ $text }”
main-suggest-kinds = Rodzaje
main-suggest-folders = Foldery
main-suggest-tags = Tagi
main-suggest-sites = Witryny
main-suggest-fields = Szczegóły
main-suggest-bookmarks = Zakładki
main-suggest-field = { $field }: { $value }
main-suggest-footer = ↑↓ wybierz · ↵ zastosuj · ⇥ uzupełnij · ⌫ usuń ostatni
main-refine-narrow = Zawęź
main-refine-more = Więcej
main-refine-less = Mniej
main-refine-clear = Wyczyść
main-display = Widok
main-display-summaries = Pokazuj podsumowania na kartach
main-display-up-next = Pokazuj pasek Na później
main-duration-minutes = { $minutes } min
main-duration-hours = { $hours } godz.
main-duration-hours-minutes = { $hours } godz. { $minutes } min

## Detail

detail-details = Szczegóły
detail-removed-from-browser = Usunięto z przeglądarki, zachowano tutaj
detail-saved = Zapisano { $date }
detail-chosen-by = Oznacza wybory modelu { $model }
detail-folders = Foldery
detail-no-folders = Brak folderów. Utwórz pierwszy na pasku bocznym.
detail-folder-undo = Cofnij: usuń z folderu
detail-folder-take-out = Usuń z folderu
detail-folder-probability = { $percent }%
detail-add-to-another-folder = Dodaj do innego folderu
detail-set-aside = Odłożone: nie pasuje do tej przestrzeni
detail-keep = Zachowaj
detail-organizing-now = Porządkowanie tej zakładki…
detail-organize-failed = Nie udało się uporządkować.
detail-try-again = Spróbuj ponownie
detail-title-and-link-only = Na razie tylko tytuł i link.
detail-read-without-ai = Odczytano tylko ze strony, bez AI.
detail-no-thumbnail = Brak miniatury na stronie
detail-page-unreachable = Nie udało się połączyć ze stroną
detail-more = Więcej
detail-tags = Tagi
detail-remove-tag = Usuń tag
detail-put-back = Przywróć
detail-add-tag = Dodaj tag
detail-add-a-tag = Dodaj tag
detail-edit = Edytuj tytuł i opis
detail-title = Tytuł
detail-add-description = Dodaj opis
detail-edit-hint = ↩ zapisz · esc anuluj
detail-in-tag-list = Na liście tagów
detail-in-tag-list-full = Na liście tagów (lista jest pełna)
detail-edit-tag-list = Edytuj listę tagów…

## Events

events-title = Wydarzenia
events-view-calendar = Kalendarz
events-view-list = Lista
events-previous-month = Poprzedni miesiąc
events-next-month = Następny miesiąc
events-today = Dzisiaj
events-upcoming = Nadchodzące
events-happening-now = Trwa teraz
events-date-not-set = Brak daty
events-past = Minione
events-nothing-on-day = Nic w tym dniu.
events-nothing-coming-up = Brak nadchodzących wydarzeń.
events-nothing-coming-up-panel = Brak nadchodzących wydarzeń. Zapisane wydarzenia pojawiają się tutaj, dopóki się nie skończą.
events-more = +{ $count } więcej

## Dates

date-today = Dzisiaj
date-yesterday = Wczoraj
date-tomorrow = Jutro
date-this-week = W tym tygodniu
date-last-week = W zeszłym tygodniu
date-all-day = Cały dzień

## Kinds

kind-article = Artykuł
kind-article-plural = Artykuły
kind-video = Wideo
kind-video-plural = Wideo
kind-movie = Film
kind-movie-plural = Filmy
kind-recipe = Przepis
kind-recipe-plural = Przepisy
kind-product = Produkt
kind-product-plural = Produkty
kind-repository = Repozytorium
kind-repository-plural = Kod
kind-book = Książka
kind-book-plural = Książki
kind-image = Obraz
kind-image-plural = Obrazy
kind-app = Aplikacja
kind-app-plural = Aplikacje
kind-podcast = Podcast
kind-podcast-plural = Podcasty
kind-place = Miejsce
kind-place-plural = Miejsca
kind-event = Wydarzenie
kind-event-plural = Wydarzenia
kind-other = Inne
kind-other-plural = Inne

## Fields

field-address = Adres
field-attendance = Udział
field-author = Autor
field-brand = Marka
field-cast = Obsada
field-category = Kategoria
field-channel = Kanał
field-city = Miasto
field-colors = Kolory
field-country = Kraj
field-cuisine = Kuchnia
field-currency = Waluta
field-diet = Dieta
field-director = Reżyseria
field-duration = Czas trwania
field-ends = Koniec
field-format = Format
field-genre = Gatunek
field-host = Prowadzący
field-language = Język
field-license = Licencja
field-maker = Twórca
field-medium = Medium
field-name = Nazwa
field-organizer = Organizator
field-owner = Właściciel
field-pages = Strony
field-platform = Platforma
field-price = Cena
field-pricing = Cennik
field-published = Opublikowano
field-publisher = Wydawca
field-rating = Ocena
field-reading-minutes = Czas czytania
field-resolution = Rozdzielczość
field-runtime = Czas trwania
field-servings = Porcje
field-show = Audycja
field-stars = Gwiazdki
field-starts = Początek
field-total-time = Łączny czas
field-venue = Miejsce
field-visible-text = Widoczny tekst
field-year = Rok

## Inks

ink-ochre = Ochra
ink-olive = Oliwka
ink-moss = Mech
ink-pine = Sosna
ink-teal = Morski
ink-sky = Błękit
ink-cobalt = Kobalt
ink-violet = Fiolet
ink-plum = Śliwka
ink-rose = Róż
ink-cocoa = Kakao
ink-slate = Łupek

## Errors

error-cant-start = Nie można uruchomić { $name }
error-quit = Zakończ
error-signed-out = Wylogowano z konta. Zaloguj się ponownie, żeby dalej synchronizować.
error-sync-unanswered = Serwer nie przyjął najnowszych zmian.
error-unlink-library = Nie udało się odłączyć biblioteki: { $error }
error-mcp-load = Nie udało się wczytać dostępu AI: { $problem }
error-mcp-save = Nie udało się zapisać dostępu AI: { $problem }
error-mcp-needs-pro = Dostęp AI wymaga { -knoa } Pro.
