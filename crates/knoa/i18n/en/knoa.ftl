## Terms
## One word per concept, used everywhere in this language.

-knoa = Knoa
-workspace = workspace
-bookmark = bookmark
-folder = folder
-tag = tag
-summary = summary
-organize = organize

## Menus

menu-about = About { $name }
menu-check-for-updates = Check for Updates…
menu-settings = Settings…
menu-services = Services
menu-hide = Hide { $name }
menu-hide-others = Hide Others
menu-show-all = Show All
menu-quit = Quit { $name }
menu-help = Help
menu-whats-new = What's New in { $name } { $version }
menu-report-issue = Report an Issue…

## Onboarding

onboarding-skip-setup = Skip setup →
onboarding-back = Back
onboarding-next = Next
onboarding-hello-title = Hi, I'm { -knoa }.
onboarding-hello-lead = You keep bookmarking things like a squirrel stocking up for winter. I find them again. I read your browser's bookmarks, work out what each one is, and keep the lot tidy.
onboarding-hello-questions = Three quick questions. Or none, if you're in a hurry.
onboarding-hello-start = Let's go
onboarding-hello-defaults = Use the defaults
onboarding-hello-have-account = Already have a { -knoa } account?
onboarding-hello-sign-in = Sign in
onboarding-browser-title = Where do your bookmarks live?
onboarding-browser-lead = Keep bookmarking the way you do. I'll notice new ones within a couple of seconds. I only read; I never touch your browser's bookmarks.
onboarding-browser-count =
    { $count ->
        [one] { $count } bookmark
       *[other] { $count } bookmarks
    }
onboarding-browser-none-found = I couldn't find a Chromium-based browser here (Helium, Chrome, Brave, Edge, Vivaldi). No worries: you can pick one later in Settings.
onboarding-browser-bring-in = Bring in { $count }
onboarding-ai-title = Want me to do the tidying?
onboarding-ai-lead = With an OpenRouter key I'll read each new page and give it a kind, a summary, tags and a folder. It's your key and your account: about 17 cents per thousand bookmarks.
onboarding-ai-key-placeholder = Paste your key: sk-or-v1-…
onboarding-ai-check-key = Check key
onboarding-ai-checking = Knocking on OpenRouter's door…
onboarding-ai-key-works = That key works.
onboarding-ai-key-missing = Paste a key first, or skip this step.
onboarding-ai-key-unknown = OpenRouter doesn't recognise that key. Double-check it?
onboarding-ai-key-check-failed = Couldn't check the key: { $error }
onboarding-ai-get-key = Get a key ↗
onboarding-ai-without-key = Without one, I still fetch images and descriptions.
onboarding-ai-skip = Skip, no AI
onboarding-library-title = Last thing: what's this library called?
onboarding-library-default-name = Personal
onboarding-library-house-rules = Any house rules for organizing?
onboarding-library-instructions-placeholder = Optional. Write it like you'd tell a friend.
onboarding-library-example-apart = Keep work and personal things apart.
onboarding-library-example-recipes = Tag recipes by cuisine.
onboarding-library-example-rust = Anything about Rust goes in a Rust folder.
onboarding-library-example-short = Short summaries, no fluff.
onboarding-library-add-example = + { $example }
onboarding-library-finish = Finish
onboarding-done-title = All set.
onboarding-done-no-browser = No browser yet, so your library starts empty. Pick one in Settings whenever you like.
onboarding-done-bringing-in = Bringing your bookmarks in…
onboarding-done-count =
    { $count ->
        [one] { $count } bookmark is in, searchable right away.
       *[other] { $count } bookmarks are in, searchable right away.
    }
onboarding-done-organize-existing =
    { $count ->
        [one] New bookmarks get organized on their own. Want me to go through the { $count } you already have too? About { $cost }.
       *[other] New bookmarks get organized on their own. Want me to go through the { $count } you already have too? About { $cost }.
    }
onboarding-done-organize = Yes, organize them
onboarding-done-open-library = Open my library

## Settings

settings-title = Settings
settings-workspaces = Workspaces
settings-new-workspace = New workspace
settings-nav-general = General
settings-nav-appearance = Appearance
settings-nav-ai = AI
settings-nav-account = Account
settings-nav-organize = Organize
settings-nav-mcp = MCP
settings-general-title = General
settings-general-about = Where { -knoa } keeps your library, how it started, and how it updates.
settings-general-library = Library
settings-general-location = Library location
settings-general-show-in-finder = Show in Finder
settings-general-setup = Setup
settings-general-run-setup-again = Run setup again
settings-general-run-setup-about = Pick browsers, AI and instructions from the start. Your bookmarks stay.
settings-general-run-setup = Run setup
settings-general-language = Language
settings-general-language-label = App and AI language
settings-general-language-about = Menus, settings and dates, and the summaries and tags the AI writes.
settings-general-language-note = Summaries and tags already written stay in their old language. Organizing your library again rewrites them.
settings-general-language-organize = Organize again…
settings-general-language-organize-confirm = Organize them all
settings-general-language-waiting = You can organize again once your account has the new language.
settings-updates = Updates
settings-updates-version = { -knoa } { $version }
settings-updates-self-built = This build doesn't update itself. Release builds from GitHub do.
settings-updates-checked-today = Last checked today at { $time }
settings-updates-checked-yesterday = Last checked yesterday at { $time }
settings-updates-checked-on = Last checked on { $date }
settings-updates-not-checked = Not checked yet
settings-updates-check-now = Check Now
settings-updates-check-automatically = Check for updates automatically
settings-updates-check-automatically-about = Once a day, in the background.
settings-updates-install-automatically = Install updates automatically
settings-updates-install-automatically-about = Download new versions and install them when you quit { -knoa }.
settings-appearance-title = Appearance
settings-appearance-about = How { -knoa } looks. Changes show right away.
settings-appearance-theme = Theme
settings-appearance-light = Light
settings-appearance-dark = Dark
settings-appearance-library = Library
settings-appearance-summaries = Show summaries on cards
settings-appearance-summaries-about = Up to three lines under each title.
settings-appearance-up-next = Show the Up next strip
settings-appearance-up-next-about = The row of things to read or watch, at the top of Everything.
settings-appearance-events = Show events as
settings-appearance-events-about = A month calendar with what's coming up, or one list. On this Mac only.
settings-reorganize-title = Organize again
settings-reorganize-folders-confirm = File them all
settings-reorganize-everything-confirm = Organize them all
settings-organize-title = Organize
settings-organize-about = Have the AI organize the bookmarks it hasn't reached yet, or go over the ones it already did.
settings-organize-scope = Bookmarks in
settings-organize-scope-all = All workspaces
settings-organize-new = Not organized yet
settings-organize-counting = Counting bookmarks…
settings-organize-none = Everything here is organized.
settings-organize-waiting =
    { $count ->
        [one] { $count } bookmark isn't organized yet
       *[other] { $count } bookmarks aren't organized yet
    }
settings-organize-ai-off = Organizing needs AI
settings-organize-ai-off-about = Connect your own key or get Pro on the AI page.
settings-organize-open-ai = Set up AI
settings-reorganize-folders-label = Folders only
settings-reorganize-folders-about = Files organized bookmarks into the folders you have now. Pages aren't read again, so it's quick and cheap.
settings-reorganize-folders-start = File again…
settings-reorganize-everything-label = Everything
settings-reorganize-everything-about = Reads each page again and redoes its summary, fields, tags and folders with the current instructions and language.
settings-reorganize-everything-start = Organize again…
settings-reorganize-kept = Folders you set or removed yourself stay as they are.
settings-reorganize-queued =
    { $count ->
        [one] Queued { $count } bookmark.
       *[other] Queued { $count } bookmarks.
    }
settings-account-synced-today = Last synced today at { $time }
settings-account-synced-yesterday = Last synced yesterday at { $time }
settings-account-synced-on = Last synced on { $date }
settings-account-not-synced = Not synced yet
settings-account-start-trial = Start free trial
settings-account-upgrade = Upgrade
settings-account-manage-subscription = Manage subscription
settings-account-usage = Usage
settings-account-usage-device = { $used } of { $limit } bookmarks
settings-account-usage-cloud = { $used } of { $limit } bookmarks, { $organized } of { $allowance } organized this month
settings-account-plan-free = Free plan.
settings-account-plan-trial = Pro trial.
settings-account-plan-pro = Pro plan.
settings-account-plan-ends = Ends { $date }.
settings-account-plan-trial-ends = Trial ends { $date }.
settings-account-plan-renews = Renews { $date }.
settings-account-organized-on-device = Organizing uses your own AI key, on this Mac.
settings-account-organized-in-cloud = { -knoa } organizes your bookmarks in the cloud.
settings-account-title = Account
settings-account-about = Sync your library with a { -knoa } account.
settings-account-unavailable = Cloud sync isn't available in this build.
settings-account-couldnt-start = It couldn't start: { $reason }
settings-account-cloud-mode = Cloud mode
settings-account-what-it-is = Cloud mode syncs your library across your devices and with { -knoa } on the web. With Pro, { -knoa } also organizes your bookmarks in the cloud, with no key of your own.
settings-account-privacy = In Cloud mode your library is kept on { -knoa }'s servers, which read your pages to organize them. It isn't end-to-end encrypted. Signed out, everything stays on this Mac.
settings-account-finish-signing-in = Finish signing in in your browser…
settings-account-finish-signing-in-about = { -knoa } opened the sign-in page in your browser.
settings-account-cancel = Cancel
settings-account-knoa-account = { -knoa } account
settings-account-sign-in-about = Sign in or create an account in your browser.
settings-account-sign-in = Sign in
settings-account-other-account = This library is synced with another { -knoa } account. Nothing syncs until you choose.
settings-account-sync-with = Sync with { $email } instead
settings-account-sync-with-this-account = Sync with this account instead
settings-account-sync-with-about = The library is merged into this account. The other account keeps its copy.
settings-account-use-this-account = Use this account
settings-account-this-account = this account
settings-account-asking = Signed in. Asking the server about the account…
settings-account-sync = Sync
settings-account-connecting = Connecting…
settings-account-up-to-date = Up to date
settings-account-syncing =
    { $count ->
        [one] Syncing { $count } change…
       *[other] Syncing { $count } changes…
    }
settings-account-offline = Offline. Changes sync when { -knoa } can reach the server.
settings-account-sync-stopped = Sync stopped: { $error }
settings-account-notice-plan-limit =
    { $unsent ->
        [one] Your plan holds { $limit } bookmarks, so { $unsent } new bookmark stayed on this Mac only.
       *[other] Your plan holds { $limit } bookmarks, so { $unsent } new bookmarks stayed on this Mac only.
    }
settings-account-notice-rejected = Some changes couldn't be synced and stayed on this Mac only.
settings-account-notice-too-large = Some changes were too large to sync and stayed on this Mac only.
settings-account-notice-allowance-used-up = This month's organizing allowance is used up. { -knoa } keeps your organizing requests and resumes them automatically when allowance becomes available.
settings-account-sign-out = Sign out
settings-account-sign-out-about = Your library stays on this Mac. Sign in again to carry on syncing.
settings-ai-access-about = Let AI apps like Claude and ChatGPT read and manage your bookmarks.
settings-ai-access-copy = Copy
settings-ai-access-url = Connector URL
settings-ai-access-default = Default access
settings-ai-access-same-as-default = Same as default
settings-ai-access-off = Off
settings-ai-access-read = Read only
settings-ai-access-full = Full access
settings-ai-access-not-linked = Available once this workspace is linked to the cloud.
settings-ai-access-saving = Saving…
settings-ai-access-loading = Loading AI access…
settings-ai-access-save-again = Save AI access again
settings-ai-access-load-again = Load AI access again
settings-ai-access-retry = Retry
settings-ai-access-needs-pro = AI access needs { -knoa } Pro
settings-mcp-title = MCP
settings-mcp-sign-in = Sign in to connect AI apps
settings-mcp-sign-in-about = AI apps reach your library through your { -knoa } account.
settings-mcp-open-account = Open Account
settings-mcp-pro-about = With Pro, AI apps like Claude and ChatGPT can search, read and organize your library.
settings-mcp-copied = Copied
settings-mcp-connect = Connect
settings-mcp-how = How to connect
settings-mcp-how-copy = Copy the connector URL.
settings-mcp-how-add = In Claude, ChatGPT or another AI app, add it as a custom connector.
settings-mcp-how-sign-in = Sign in to { -knoa } when the app asks.
settings-mcp-how-about = A connected AI app can read your bookmarks, including text saved from their pages, in every workspace it has access to. Changes here apply from its next request.
settings-mcp-access = Access
settings-mcp-default-off = AI apps cannot access workspaces that use the default.
settings-mcp-default-read = AI apps can search and read bookmarks in workspaces that use the default.
settings-mcp-default-full = AI apps can also add, edit and trash bookmarks, and manage folders and workspaces.
settings-ai-title = AI
settings-ai-about = { -knoa } works without AI too. With it, new bookmarks get a kind, a summary, tags and folders.
settings-ai-provider = Provider
settings-ai-cloud-note = With Pro, { -knoa } organizes your bookmarks in the cloud. A key you connected stays on this Mac for when you're not signed in.
settings-ai-key = OpenRouter key
settings-ai-key-about = Your own key; requests go from this Mac straight to OpenRouter.
settings-ai-connect = Connect
settings-ai-checking = Checking the key…
settings-ai-key-connected = Connected. New bookmarks get organized from now on.
settings-ai-key-failed = That didn't work: { $error }.
settings-ai-no-key = No key yet?
settings-ai-no-key-about = A few dollars of credit organizes thousands of bookmarks.
settings-ai-get-key = Get a key
settings-ai-connected = Connected
settings-ai-disconnect = Disconnect
settings-ai-openrouter-says = OpenRouter says: { $problem }. Check your key or credits.
settings-ai-models = Models
settings-ai-included-in-pro = Included in Pro, within its monthly allowance.
settings-ai-cost = About { $cost } to do them all.
settings-ai-organize = Organize
settings-ai-pro-title = Let { -knoa } do it with Pro
settings-ai-pro-pitch = { -knoa } organizes your bookmarks in the cloud. No key to set up, no bill per request, and your library syncs across your devices.
settings-ai-pro-get = Get Pro
settings-ai-pro-upgrade = Upgrade
settings-ai-pro-have = I have Pro
settings-ai-usage = Usage
settings-ai-usage-none = No AI requests yet.
settings-ai-usage-recent =
    { $days ->
        [one] Last { $days } day
       *[other] Last { $days } days
    }
settings-ai-usage-all-time = All time
settings-ai-usage-by-model =
    { $days ->
        [one] By model, last { $days } day
       *[other] By model, last { $days } days
    }
settings-ai-usage-records = { $provider }'s own records
settings-ai-usage-records-about = Every request, with the exact amount charged.
settings-ai-usage-open = Open
settings-ai-usage-requests =
    { $count ->
        [one] { $count } request
       *[other] { $count } requests
    }
settings-ai-usage-tokens = { $input } tokens in, { $output } out
settings-ai-usage-unmetered =
    { $count ->
        [one] { $count } without token counts
       *[other] { $count } without token counts
    }
settings-ai-usage-unpriced =
    { $count ->
        [one] { $count } without a price
       *[other] { $count } without a price
    }
settings-workspace-new = New workspace
settings-workspace-new-about = A separate library, like Work or Personal, with its own browsers and folders.
settings-workspace-shown-now = Shown now
settings-workspace-show-it = Show it
settings-workspace-name-and-color = Name and color
settings-workspace-name = Name
settings-workspace-name-too-long = A name can be at most { $max } characters.
settings-workspace-color = Color
settings-workspace-no-browser = No Chromium-based browser found. { -knoa } reads Helium, Chrome, Brave, Edge, Vivaldi and Chromium.
settings-workspace-browsers-note = With none on, links are only added by hand (⌘N). One browser can feed several workspaces.
settings-workspace-open-in = Open bookmarks in
settings-workspace-browsers = Browsers
settings-workspace-open-in-about = Where this workspace's links open.
settings-workspace-organize-again-about = After you change instructions or tags, file or organize this workspace's bookmarks again.
settings-workspace-open-organize = Organize…
settings-workspace-default-browser = Default browser
settings-workspace-by-hand = Links added by hand only.
settings-workspace-synced-from-one = Synced from { $browser }.
settings-workspace-synced-from-many = Synced from { $browsers } and { $last }.
settings-workspace-organizing = Organizing
settings-workspace-instructions = Instructions
settings-workspace-instructions-about = How the AI should sort, tag and summarize what lands here.
settings-workspace-instructions-too-long = Instructions can be at most { $max } characters.
settings-workspace-screening = Only keep bookmarks that fit
settings-workspace-screening-about = Off: everything from the browser lands here. On: the AI sets aside what doesn't fit, and you can keep it anyway.
settings-workspace-guideline = What belongs here
settings-workspace-guideline-too-long = A guideline can be at most { $max } characters.
settings-workspace-name-placeholder = Work, Personal…
settings-workspace-instructions-placeholder = e.g. Tag recipes by cuisine. Short summaries, no fluff.
settings-workspace-guideline-placeholder = e.g. Only things about work: engineering, design and our clients.
settings-workspace-create = Create workspace
settings-workspace-delete = Delete
settings-workspace-delete-label = Delete this workspace
settings-workspace-delete-about = Its bookmarks and folders go with it. Your browsers keep their bookmarks.
settings-workspace-delete-button = Delete…
settings-workspace-delete-confirm = Delete for good
settings-tags-title = Tag list
settings-tags-label = Tags the AI should use
settings-tags-about = When one of these fits a bookmark, the AI uses it instead of a word of its own. A short hint tells it when one fits. You can also right-click a tag on a bookmark to list it.
settings-tags-add-placeholder = Add a tag
settings-tags-hint-placeholder = When to use it (optional)
settings-tags-remove = Remove from list
settings-tags-only-listed = Only use tags from this list
settings-tags-only-listed-about = Off: the AI adds its own tags for what the list doesn't cover. On: bookmarks get only listed tags, or none when nothing fits.
settings-tags-only-listed-about-empty = Add a tag first. Off: the AI adds its own tags for what the list doesn't cover. On: bookmarks get only listed tags, or none when nothing fits.
settings-tags-note = Bookmarks organized from now on get these tags. To bring them to the ones already organized, organize again.
settings-tags-count = { $count } of { $max }

## Sidebar

sidebar-status-ai-needs-attention = AI needs attention
sidebar-status-fix = Fix
sidebar-sync-stopped = Sync stopped
sidebar-retry = Retry
sidebar-sync-paused = Sync paused
sidebar-status-changes-not-synced = Changes didn't sync
sidebar-status-view = View
sidebar-status-plan-limit = Plan limit reached
sidebar-status-upgrade = Upgrade
sidebar-status-organizing = Organizing { $current } of { $total }…
sidebar-status-ai-off = AI is off
sidebar-status-turn-on = Turn on
sidebar-status-offline = Offline. Changes sync later.
sidebar-syncing = Syncing…
sidebar-sync-offline =
    { $count ->
        [0] Offline
        [one] Offline · { $count } change waiting
       *[other] Offline · { $count } changes waiting
    }
sidebar-synced-just-now = Synced just now
sidebar-synced-minutes-ago =
    { $minutes ->
        [one] Synced { $minutes } min ago
       *[other] Synced { $minutes } min ago
    }
sidebar-synced-hours-ago =
    { $hours ->
        [one] Synced { $hours } hr ago
       *[other] Synced { $hours } hr ago
    }
sidebar-account-settings = Account settings…
sidebar-open-web = Open { -knoa } on the web
sidebar-sign-out = Sign out
sidebar-sign-in = Sign in…
sidebar-finish-in-browser = Finish in browser
sidebar-cancel = Cancel
sidebar-plan-pro = { -knoa } Pro
sidebar-plan-free = Free plan
sidebar-signed-in = Signed in
sidebar-manage-plan = Manage plan…
sidebar-upgrade-to-pro = Upgrade to Pro
sidebar-this-mac = This Mac
sidebar-knoa-account = { -knoa } account
sidebar-place-everything = Everything
sidebar-place-gallery = Gallery
sidebar-place-events = Events
sidebar-place-up-next = Up next
sidebar-place-unsorted = Unsorted
sidebar-place-not-organized = Not organized
sidebar-place-set-aside = Set aside
sidebar-place-trash = Trash
sidebar-settings = Settings
sidebar-folders = Folders
sidebar-new-folder = New folder
sidebar-folder-new-subfolder = New subfolder
sidebar-folder-edit = Edit folder…
sidebar-folder-delete = Delete…
sidebar-folder-move-to = Move to
sidebar-folder-top-level = Top level
sidebar-add-link = Add link
sidebar-workspaces-title = Workspaces
sidebar-workspaces-new = New workspace
sidebar-workspaces-by-hand = Links added by hand
sidebar-workspaces-from = From { $browsers }

## Main

main-blocked-browsers =
    { $count ->
        [one] { $browsers } is installed, but macOS keeps its bookmarks private. Give { -knoa } Full Disk Access, then restart { -knoa }.
       *[other] { $browsers } are installed, but macOS keeps their bookmarks private. Give { -knoa } Full Disk Access, then restart { -knoa }.
    }
main-open-privacy-settings = Open Privacy settings
main-cancel = Cancel
main-delete = Delete
main-save = Save
main-folder-new-title = New folder
main-folder-edit-title = Edit folder
main-folder-name-placeholder = Folder name
main-folder-description-placeholder = What belongs here? The AI uses this to file bookmarks.
main-folder-name-too-long = A name can be at most { $max } characters.
main-folder-description-too-long = A description can be at most { $max } characters.
main-folder-auto-assign = Let the AI add bookmarks
main-folder-auto-assign-about = New bookmarks that fit go here on their own.
main-glyph-read = Reading
main-glyph-article = Articles
main-glyph-code = Code
main-glyph-write = Writing
main-glyph-work = Work
main-glyph-tag = Deals
main-glyph-bag = Shopping
main-glyph-plane = Travel
main-glyph-cup = Food
main-glyph-music = Music
main-glyph-play = Video
main-glyph-photo = Photos
main-glyph-bulb = Ideas
main-glyph-heart = Health
main-glyph-home = Home
main-glyph-star = Favorites
main-glyph-clock = Later
main-glyph-flask = Science
main-glyph-globe = Web
main-glyph-person = People
main-glyph-box = Archive
main-glyph-inbox = Inbox
main-glyph-leaf = Nature
main-glyph-bolt = Quick
main-glyph-lock = Private
main-glyph-pin = Places
main-glyph-chat = Threads
main-glyph-game = Games
main-folder-icon-none = Folder
main-folder-icon-emoji = Emoji
main-folder-parent = Parent
main-folder-parent-none = None
main-folder-emoji-placeholder = Type or paste
main-folder-any-emoji = Any emoji
main-folder-custom-color = Custom
main-delete-folder-title = Delete “{ $name }”?
main-delete-folder-detail =
    { $count ->
        [0] Its bookmarks stay in your library.
        [one] Its subfolder is deleted with it. Their bookmarks stay in your library.
       *[other] Its { $count } subfolders are deleted with it. Their bookmarks stay in your library.
    }
main-open = Open
main-open-in-background = Open in background
main-copy-link = Copy link
main-copied = Copied
main-show-details = Show details
main-view-picture = View picture
main-copy-image = Copy image
main-save-image = Save image…
main-add-to-up-next = Add to Up next
main-remove-from-up-next = Remove from Up next
main-add-to-folder = Add to folder
main-remove-from-folder = Remove from { $folder }
main-refresh-thumbnail = Refresh thumbnail
main-refreshing-thumbnail = Refreshing thumbnail…
main-organize = Organize
main-organize-again = Organize again
main-organizing = Organizing…
main-move-to-trash = Move to Trash
main-restore = Restore
main-delete-now = Delete now
main-open-link = Open link
main-card-failed = Couldn't organize
main-card-not-organized = Not organized yet
main-empty-no-events-match = No events match
main-empty-no-events-match-body = Try fewer words, or a date filter like starts>2026-12-01.
main-empty-nothing-matches = Nothing matches
main-empty-nothing-matches-body = Try fewer words, or filters like kind:movie, site:github.com or #tag.
main-empty-trash = Trash is empty
main-empty-trash-body = Bookmarks you delete wait here for 30 days.
main-empty-caught-up = All caught up
main-empty-caught-up-body = Every bookmark here has been organized.
main-empty-no-pictures = No pictures here
main-empty-no-pictures-body = Links straight to an image file show up here.
main-empty-no-events = No events here
main-empty-no-events-body = Pages for concerts, talks and meetups show up here by date.
main-empty-nothing-yet = Nothing here yet
main-empty-nothing-yet-synced-body = Bookmark something in your browser and it lands here a moment later.
main-empty-nothing-yet-unsynced-body = Add a link with ⌘N, or pick a browser for this workspace from the workspace menu.
main-empty-nothing-yet-body = Bookmarks show up here as you add them.
main-up-next = Up next
main-up-next-count =
    { $count ->
        [one] · { $count } to read and watch
       *[other] · { $count } to read and watch
    }
main-best-matches = Best matches
main-place-gallery = Gallery
main-place-events = Events
main-place-up-next = Up next
main-place-unsorted = Unsorted
main-place-not-organized = Not organized
main-place-set-aside = Set aside
main-place-trash = Trash
main-search-found =
    { $count ->
        [one] { $count } found
       *[other] { $count } found
    }
main-kind-all = All
main-kind-chip = { $kind } { $count }
main-trash-empty = Empty Trash
main-trash-delete-count =
    { $count ->
        [one] Delete { $count } for good
       *[other] Delete { $count } for good
    }
main-trash-about = Bookmarks in Trash are deleted for good after 30 days.
main-trash-days-left =
    { $days ->
        [1] In Trash. Deleted for good within a day.
       *[other] In Trash. Deleted for good in { $days } days.
    }
main-add-link-title = Add a link to { $workspace }
main-add-link-not-a-link = That doesn't look like a web address.
main-add-link-add = Add
main-image-file-name = Image
main-viewer-position = { $position } of { $count }
main-viewer-open-original = Open original
main-viewer-close = Close
main-viewer-image-copied = Image copied
main-viewer-copy-failed = Couldn't copy the image
main-viewer-image-saved = Image saved
main-viewer-save-failed = Couldn't save the image
main-search-placeholder = Just start typing to search
main-view-options-title = View
main-view-options-summaries = Show summaries on cards
main-view-options-up-next = Show the Up next strip
main-view-options-about = Cards are grouped by date. Searching ranks by best match.
main-duration-minutes = { $minutes } min
main-duration-hours = { $hours }h
main-duration-hours-minutes = { $hours }h { $minutes }m

## Detail

detail-details = Details
detail-removed-from-browser = Removed from the browser, kept here
detail-saved = Saved { $date }
detail-chosen-by = Marks what { $model } chose
detail-folders = Folders
detail-no-folders = No folders yet. Make one in the sidebar.
detail-folder-undo = Undo: take it out
detail-folder-take-out = Take it out
detail-folder-probability = { $percent }%
detail-add-to-another-folder = Add to another folder
detail-set-aside = Set aside: doesn't fit this workspace
detail-keep = Keep
detail-organizing-now = Organizing this one now…
detail-organize-failed = Organizing didn't work out.
detail-try-again = Try again
detail-title-and-link-only = Only the title and link so far.
detail-read-without-ai = Read from the page only, without AI.
detail-no-thumbnail = No thumbnail on the page
detail-page-unreachable = Couldn't reach the page
detail-more = More
detail-tags = Tags
detail-remove-tag = Remove tag
detail-put-back = Put back
detail-add-tag = Add tag
detail-add-a-tag = Add a tag
detail-in-tag-list = In tag list
detail-in-tag-list-full = In tag list (list is full)
detail-edit-tag-list = Edit tag list…

## Events

events-title = Events
events-view-calendar = Calendar
events-view-list = List
events-previous-month = Previous month
events-next-month = Next month
events-today = Today
events-upcoming = Upcoming
events-happening-now = Happening now
events-date-not-set = Date not set
events-past = Past
events-nothing-on-day = Nothing on this day.
events-nothing-coming-up = Nothing coming up.
events-nothing-coming-up-panel = Nothing coming up. Events you save show here until they end.
events-more = +{ $count } more

## Dates

date-today = Today
date-yesterday = Yesterday
date-tomorrow = Tomorrow
date-this-week = This week
date-last-week = Last week
date-all-day = All day

## Kinds

kind-article = Article
kind-article-plural = Articles
kind-video = Video
kind-video-plural = Videos
kind-movie = Movie
kind-movie-plural = Movies
kind-recipe = Recipe
kind-recipe-plural = Recipes
kind-product = Product
kind-product-plural = Products
kind-repository = Repository
kind-repository-plural = Code
kind-book = Book
kind-book-plural = Books
kind-image = Image
kind-image-plural = Images
kind-app = App
kind-app-plural = Apps
kind-podcast = Podcast
kind-podcast-plural = Podcasts
kind-place = Place
kind-place-plural = Places
kind-event = Event
kind-event-plural = Events
kind-other = Other
kind-other-plural = Other

## Fields

field-address = Address
field-attendance = Attendance
field-author = Author
field-brand = Brand
field-cast = Cast
field-category = Category
field-channel = Channel
field-city = City
field-colors = Colors
field-country = Country
field-cuisine = Cuisine
field-currency = Currency
field-diet = Diet
field-director = Director
field-duration = Duration
field-ends = Ends
field-format = Format
field-genre = Genre
field-host = Host
field-language = Language
field-license = License
field-maker = Maker
field-medium = Medium
field-name = Name
field-organizer = Organizer
field-owner = Owner
field-pages = Pages
field-platform = Platform
field-price = Price
field-pricing = Pricing
field-published = Published
field-publisher = Publisher
field-rating = Rating
field-reading-minutes = Reading minutes
field-resolution = Resolution
field-runtime = Runtime
field-servings = Servings
field-show = Show
field-stars = Stars
field-starts = Starts
field-total-time = Total time
field-venue = Venue
field-visible-text = Visible text
field-year = Year

## Inks

ink-ochre = Ochre
ink-olive = Olive
ink-moss = Moss
ink-pine = Pine
ink-teal = Teal
ink-sky = Sky
ink-cobalt = Cobalt
ink-violet = Violet
ink-plum = Plum
ink-rose = Rose
ink-cocoa = Cocoa
ink-slate = Slate

## Errors

error-cant-start = { $name } can't start
error-quit = Quit
error-signed-out = You were signed out. Sign in again to keep syncing.
error-sync-unanswered = The server didn't take the latest changes.
error-unlink-library = Couldn't unlink the library: { $error }
error-mcp-load = Couldn't load AI access: { $problem }
error-mcp-save = Couldn't save AI access: { $problem }
error-mcp-needs-pro = AI access needs { -knoa } Pro.
