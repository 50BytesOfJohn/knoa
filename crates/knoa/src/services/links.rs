//! Opening bookmarks in the browser their workspace picked.

use gpui::{App, AppContext as _, AsyncApp};
use knoa_browser::ProfileId;

/// Opens `url` in the profile a workspace picked (see [`knoa_core::Workspace::opens_in`]),
/// or the default browser if it picked none. If the profile can't be opened, the default
/// browser is used instead, so a click never does nothing.
pub fn open(url: &str, opens_in: Option<&str>, cx: &mut App) {
    let profile = opens_in.and_then(ProfileId::parse);
    let Some(profile) = profile else {
        cx.open_url(url);
        return;
    };
    let url = url.to_owned();
    let opening = cx.background_spawn({
        let url = url.clone();
        async move { knoa_browser::open_in_profile(&url, &profile) }
    });
    cx.spawn(async move |cx: &mut AsyncApp| {
        if let Err(err) = opening.await {
            tracing::warn!("{err}; opening in the default browser");
            cx.update(|cx| cx.open_url(&url));
        }
    })
    .detach();
}
