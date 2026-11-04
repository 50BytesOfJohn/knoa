//! The tokio runtime for page fetches and AI calls (ADR 0006). GPUI tasks await its
//! `JoinHandle`s; nothing on the UI thread blocks on the network.

use gpui::{App, Global};
use tokio::{runtime::Runtime, task::JoinHandle};

pub struct Network {
    runtime: Runtime,
}

impl Global for Network {}

impl Network {
    pub fn init(cx: &mut App) -> anyhow::Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("knoa-network")
            .enable_all()
            .build()?;
        cx.set_global(Network { runtime });
        Ok(())
    }

    pub fn handle(cx: &App) -> tokio::runtime::Handle {
        cx.global::<Network>().runtime.handle().clone()
    }

    pub fn spawn<F>(cx: &App, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        cx.global::<Network>().runtime.spawn(future)
    }
}
