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

## Errors

error-cant-start = { $name } can't start
error-quit = Quit
