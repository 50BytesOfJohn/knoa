//! How long the UI's transitions take, and macOS's Reduce Motion setting.
//!
//! GPUI's `with_animation` already draws a still frame when `App::reduce_motion` is set:
//! the end of a one-shot animation and the start of a loop. GPUI doesn't read the system
//! setting, so it is handed over here, at launch and whenever it changes.

use std::{ptr::NonNull, time::Duration};

use block2::RcBlock;
use futures::{StreamExt as _, channel::mpsc};
use gpui::App;
use objc2::{
    rc::Retained,
    runtime::{NSObjectProtocol, ProtocolObject},
};
use objc2_app_kit::{NSWorkspace, NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification};
use objc2_foundation::{NSNotification, NSOperationQueue};

/// A folder's chevron turning as it opens or closes.
pub const CHEVRON: Duration = Duration::from_millis(150);
/// A menu fading and sliding in from its button.
pub const MENU_OPEN: Duration = Duration::from_millis(120);
/// How far a menu slides as it opens.
pub const MENU_SLIDE: f32 = 4.;
/// The sidebar's status card fading in.
pub const CARD_FADE: Duration = Duration::from_millis(160);
/// The organizing bar moving to its new length.
pub const BAR: Duration = Duration::from_millis(200);
/// One turn of the dashed circle while the AI organizes.
pub const SLOW_TURN: Duration = Duration::from_secs(8);
/// Decorative loops redraw at this rate rather than every display frame.
pub const LOOP_FPS: f32 = 30.;

/// Follows the system's Reduce Motion setting for the rest of the app's life.
pub fn follow_system_setting(cx: &mut App) {
    cx.set_reduce_motion(system_reduces_motion());
    let (changed, mut changes) = mpsc::unbounded::<()>();
    let observer = observe_display_options(changed);
    cx.spawn(async move |cx| {
        // Never removed: the setting is followed for the app's whole life.
        let _observer = observer;
        while changes.next().await.is_some() {
            cx.update(|cx| cx.set_reduce_motion(system_reduces_motion()));
        }
    })
    .detach();
}

/// Sends on `changed` whenever an accessibility display option, such as Reduce Motion,
/// changes. The returned token identifies the observer.
#[expect(
    unsafe_code,
    reason = "GPUI doesn't wrap AppKit's notification observers"
)]
fn observe_display_options(
    changed: mpsc::UnboundedSender<()>,
) -> Retained<ProtocolObject<dyn NSObjectProtocol>> {
    let block = RcBlock::new(move |_: NonNull<NSNotification>| {
        let _ = changed.unbounded_send(());
    });
    let center = NSWorkspace::sharedWorkspace().notificationCenter();
    let queue = NSOperationQueue::mainQueue();
    // SAFETY: the name is AppKit's own constant, defined for the whole process.
    let name = unsafe { NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification };
    // SAFETY: no object filter is given, the main queue runs the block on the main thread,
    // and the block only holds a channel sender, which is `Send + Sync`.
    unsafe {
        center.addObserverForName_object_queue_usingBlock(Some(name), None, Some(&queue), &block)
    }
}

fn system_reduces_motion() -> bool {
    NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
}
