//! Ordered credential access: Keychain in release builds, a private file in dev builds.

mod dev_file;
mod queue;

use std::path::PathBuf;

use gpui::{App, AppContext as _, Global, Task};

use self::queue::{Operation, Queue};
use crate::channel::Channel;

pub struct Credentials {
    queue: Queue,
    _worker: Task<()>,
}

impl Global for Credentials {}

/// The Keychain item's account name; the URL identifies the provider.
const ACCOUNT: &str = "api-key";

impl Credentials {
    pub fn init(dev_file: PathBuf, cx: &mut App) {
        let path = (Channel::current() == Channel::Dev).then_some(dev_file);
        let (queue, mut requests) = Queue::new();
        let worker = cx.spawn(async move |cx| {
            use futures::StreamExt as _;

            while let Some(request) = requests.next().await {
                if let Some(path) = &path {
                    let path = path.clone();
                    cx.background_spawn(async move { request.on_dev_file(&path) })
                        .await;
                } else {
                    on_keychain(request, cx).await;
                }
            }
        });
        cx.set_global(Credentials {
            queue,
            _worker: worker,
        });
    }

    pub fn read(url: &str, cx: &App) -> Task<anyhow::Result<Option<String>>> {
        let reply = cx
            .global::<Credentials>()
            .queue
            .request(url, Operation::Read);
        cx.background_spawn(async move { reply.await? })
    }

    pub fn write(url: &str, secret: &str, cx: &App) -> Task<anyhow::Result<()>> {
        Self::mutate(url, Operation::Write(secret.to_owned()), cx)
    }

    pub fn delete(url: &str, cx: &App) -> Task<anyhow::Result<()>> {
        Self::mutate(url, Operation::Delete, cx)
    }

    fn mutate(url: &str, operation: Operation, cx: &App) -> Task<anyhow::Result<()>> {
        // Enqueue now, not when the caller awaits: deletion is a barrier for prior writes.
        let reply = cx.global::<Credentials>().queue.request(url, operation);
        cx.background_spawn(async move { reply.await?.map(|_| ()) })
    }
}

async fn on_keychain(request: queue::Request, cx: &gpui::AsyncApp) {
    let task = cx.update(|cx| match &request.operation {
        Operation::Read => {
            let task = cx.read_credentials(&request.url);
            cx.background_spawn(async move {
                Ok(task
                    .await?
                    .map(|(_, secret)| String::from_utf8_lossy(&secret).into_owned()))
            })
        }
        Operation::Write(secret) => {
            let task = cx.write_credentials(&request.url, ACCOUNT, secret.as_bytes());
            cx.background_spawn(async move { task.await.map(|()| None) })
        }
        Operation::Delete => {
            let task = cx.delete_credentials(&request.url);
            cx.background_spawn(async move { task.await.map(|()| None) })
        }
    });
    let _ = request.reply.send(task.await);
}
