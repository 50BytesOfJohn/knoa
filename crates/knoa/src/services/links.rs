//! Opening bookmarks in the browser their workspace picked.

use gpui::{App, AppContext as _, AsyncApp};
use knoa_browser::{Activation, ProfileId};

/// Opens `url` in the profile a workspace picked (see [`knoa_core::Workspace::opens_in`]),
/// or the default browser if it picked none. If that fails, the default browser is asked
/// the usual way instead, so a click never does nothing.
pub fn open(url: &str, opens_in: Option<&str>, activation: Activation, cx: &mut App) {
    let profile = opens_in.and_then(ProfileId::parse);
    if profile.is_none() && activation == Activation::Foreground {
        cx.open_url(url);
        return;
    }
    let url = url.to_owned();
    let opening = cx.background_spawn({
        let url = url.clone();
        async move {
            match profile {
                Some(profile) => knoa_browser::open_in_profile(&url, &profile, activation),
                None => knoa_browser::open_in_default(&url, activation),
            }
        }
    });
    cx.spawn(async move |cx: &mut AsyncApp| {
        if let Err(err) = opening.await {
            tracing::warn!("{err}; opening in the default browser");
            cx.update(|cx| cx.open_url(&url));
        }
    })
    .detach();
}
