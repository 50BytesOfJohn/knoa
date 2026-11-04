//! Connecting the configured AI provider: reading its key and handing it to the organizer.

use gpui::{App, AppContext as _, Task, TaskExt as _};
use knoa_ai::{Provider, ProviderConfig, ProviderKind};

use crate::{
    services::{credentials::Credentials, network::Network, organizer::Organizer},
    state::settings::SettingsFile,
};

/// Connects the provider from settings, or turns AI off when there is none or no key.
pub fn connect(cx: &mut App) {
    let Some(config) = SettingsFile::get(cx).ai.clone() else {
        set_provider(None, cx);
        return;
    };
    let read = Credentials::read(config.kind.credential_url(), cx);
    cx.spawn(async move |cx| {
        let key = match read.await {
            Ok(Some(key)) => key,
            Ok(None) => {
                tracing::warn!("AI is configured but no API key is stored");
                return;
            }
            Err(err) => {
                tracing::error!("can't read the API key: {err:#}");
                return;
            }
        };
        cx.update(|cx| match Provider::new(&config, key) {
            Ok(provider) => set_provider(Some(provider), cx),
            Err(err) => tracing::error!("can't set up {}: {err}", config.kind.name()),
        });
    })
    .detach();
}

/// Checks a key with a free request before it is saved.
pub fn check_key(kind: ProviderKind, key: String, cx: &App) -> Task<Result<(), knoa_ai::Error>> {
    let handle = Network::spawn(cx, async move {
        let provider = Provider::new(&ProviderConfig::default_for(kind), key)?;
        provider.check_key().await
    });
    cx.background_spawn(async move {
        handle
            .await
            .unwrap_or_else(|err| Err(knoa_ai::Error::BadAnswer(err.to_string())))
    })
}

/// Stores the key, turns AI on with the provider's defaults, and connects.
pub fn enable(kind: ProviderKind, key: &str, cx: &mut App) -> Task<anyhow::Result<()>> {
    let write = Credentials::write(kind.credential_url(), key, cx);
    cx.spawn(async move |cx| {
        write.await?;
        cx.update(|cx| {
            SettingsFile::update(cx, |s| s.ai = Some(ProviderConfig::default_for(kind)));
            connect(cx);
        });
        Ok(())
    })
}

/// Turns AI off and forgets the key.
pub fn disable(cx: &mut App) {
    if let Some(config) = SettingsFile::get(cx).ai.clone() {
        Credentials::delete(config.kind.credential_url(), cx).detach_and_log_err(cx);
    }
    SettingsFile::update(cx, |s| s.ai = None);
    set_provider(None, cx);
}

fn set_provider(provider: Option<Provider>, cx: &mut App) {
    Organizer::entity(cx).update(cx, |organizer, cx| organizer.set_provider(provider, cx));
}
