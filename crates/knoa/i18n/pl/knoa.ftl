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
        [one] New bookmarks get organized on their own. Want me to go through the { $count } you already have too? About ${ $cost }.
       *[other] New bookmarks get organized on their own. Want me to go through the { $count } you already have too? About ${ $cost }.
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
settings-reorganize-about = Folders only files organized bookmarks into the current folders, without reading pages or asking the chat model again. Everything also redoes summaries, fields and tags with the current instructions. Folders you set or removed stay as they are.
settings-reorganize-all-title = Organize every workspace again
settings-reorganize-all-about = The same for all workspaces at once, after you've changed folders in several.
settings-reorganize-folders = Folders only…
settings-reorganize-folders-confirm = File them all
settings-reorganize-everything = Everything…
settings-reorganize-everything-confirm = Organize them all
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

## Detail

detail-details = Details
detail-removed-from-browser = Removed from the browser, kept here
detail-saved = Saved { $date }
detail-chosen-by = Marks what { $model } chose

## Events

events-title = Events
events-view-calendar = Calendar
events-view-list = List
events-previous-month = Previous month
events-next-month = Next month
events-today = Today

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
