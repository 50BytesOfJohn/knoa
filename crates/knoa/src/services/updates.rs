//! Software updates through Sparkle, which release bundles embed in `Contents/Frameworks`.
//! Sparkle owns the schedule, the update window and the install; Knoa only starts it and
//! exposes its two preferences. A build without the framework (`cargo run`) has no updater.

#![expect(
    unsafe_code,
    reason = "Sparkle is an Objective-C framework, reached through objc2 messages"
)]

use gpui::{App, Global};
use jiff::Timestamp;
use objc2::{
    msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyClass, AnyObject},
};
use objc2_foundation::{NSBundle, NSDate, NSString};

use crate::channel::Channel;

pub struct Updates {
    /// The `SPUStandardUpdaterController`, which shows Sparkle's standard windows. It owns
    /// `updater` and has to live as long as the app.
    _controller: Retained<AnyObject>,
    /// The controller's `SPUUpdater`.
    updater: Retained<AnyObject>,
}

impl Global for Updates {}

impl Updates {
    /// Starts Sparkle when the bundle embeds it. From then on it checks on its own schedule.
    pub fn init(cx: &mut App) {
        if Channel::current() != Channel::Release {
            return;
        }
        if let Some(updates) = start() {
            cx.set_global(updates);
        } else {
            tracing::info!("Sparkle isn't in this build; updates are off");
        }
    }

    pub fn get(cx: &App) -> Option<&Updates> {
        cx.try_global::<Updates>()
    }

    /// Checks now and shows the result, even when there's nothing new.
    pub fn check_now(&self) {
        // SAFETY: -[SPUUpdater checkForUpdates] takes nothing and returns nothing.
        unsafe { msg_send![&*self.updater, checkForUpdates] }
    }

    pub fn checks_automatically(&self) -> bool {
        // SAFETY: a BOOL property getter on SPUUpdater.
        unsafe { msg_send![&*self.updater, automaticallyChecksForUpdates] }
    }

    pub fn set_checks_automatically(&self, on: bool) {
        // SAFETY: the matching BOOL setter; Sparkle saves it in the user defaults.
        unsafe { msg_send![&*self.updater, setAutomaticallyChecksForUpdates: on] }
    }

    pub fn installs_automatically(&self) -> bool {
        // SAFETY: a BOOL property getter on SPUUpdater.
        unsafe { msg_send![&*self.updater, automaticallyDownloadsUpdates] }
    }

    /// On: updates download in the background and install when Knoa quits. Off: Sparkle
    /// asks first, showing the release notes.
    pub fn set_installs_automatically(&self, on: bool) {
        // SAFETY: the matching BOOL setter; Sparkle saves it in the user defaults.
        unsafe { msg_send![&*self.updater, setAutomaticallyDownloadsUpdates: on] }
    }

    pub fn last_checked(&self) -> Option<Timestamp> {
        // SAFETY: -[SPUUpdater lastUpdateCheckDate] returns a nullable NSDate.
        let date: Option<Retained<NSDate>> =
            unsafe { msg_send![&*self.updater, lastUpdateCheckDate] };
        Timestamp::from_second(date?.timeIntervalSince1970() as i64).ok()
    }
}

fn start() -> Option<Updates> {
    let frameworks = NSBundle::mainBundle().privateFrameworksPath()?;
    let path = NSString::from_str(&format!("{frameworks}/Sparkle.framework"));
    let sparkle = NSBundle::bundleWithPath(&path)?;
    // SAFETY: loads the framework the bundle ships, signed with the app. Loading runs
    // only its own initializers.
    if !unsafe { sparkle.load() } {
        tracing::warn!("couldn't load {path}; updates are off");
        return None;
    }
    let class = AnyClass::get(c"SPUStandardUpdaterController")?;
    let none: Option<&AnyObject> = None;
    // SAFETY: -[SPUStandardUpdaterController initWithStartingUpdater:updaterDelegate:
    // userDriverDelegate:] takes a BOOL and two nullable delegates, and is called on the
    // main thread, where GPUI runs app code.
    let controller: Retained<AnyObject> = unsafe {
        let allocated: Allocated<AnyObject> = msg_send![class, alloc];
        msg_send![allocated, initWithStartingUpdater: true, updaterDelegate: none, userDriverDelegate: none]
    };
    // SAFETY: -[SPUStandardUpdaterController updater] returns the non-null SPUUpdater.
    let updater: Retained<AnyObject> = unsafe { msg_send![&*controller, updater] };
    tracing::info!("Sparkle started");
    Some(Updates {
        _controller: controller,
        updater,
    })
}
