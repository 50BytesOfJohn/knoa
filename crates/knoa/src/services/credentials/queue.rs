//! Invocation order for credential reads, writes and deletion barriers.

use std::path::Path;

use futures::channel::{mpsc, oneshot};

use super::dev_file;

type Answer = anyhow::Result<Option<String>>;

pub(super) enum Operation {
    Read,
    Write(String),
    Delete,
}

pub(super) struct Request {
    pub url: String,
    pub operation: Operation,
    pub reply: oneshot::Sender<Answer>,
}

pub(super) struct Queue(mpsc::UnboundedSender<Request>);

impl Queue {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Request>) {
        let (sender, receiver) = mpsc::unbounded();
        (Self(sender), receiver)
    }

    pub fn request(&self, url: &str, operation: Operation) -> oneshot::Receiver<Answer> {
        let (reply, received) = oneshot::channel();
        let _ = self.0.unbounded_send(Request {
            url: url.to_owned(),
            operation,
            reply,
        });
        received
    }
}

impl Request {
    pub fn on_dev_file(self, path: &Path) {
        let result = dev_file::apply(path, &self.url, self.operation);
        let _ = self.reply.send(result);
    }
}

#[cfg(test)]
mod tests {
    use futures::{StreamExt as _, executor::block_on};

    use super::*;

    #[test]
    fn deletion_follows_older_writes_even_when_their_callers_stop_waiting() {
        block_on(async {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("credentials.json");
            let (queue, mut requests) = Queue::new();
            drop(queue.request("cloud", Operation::Write("old-refresh".into())));
            drop(queue.request("ai", Operation::Write("keep-this-key".into())));
            let deleted = queue.request("cloud", Operation::Delete);
            drop(queue);
            while let Some(request) = requests.next().await {
                request.on_dev_file(&path);
            }
            deleted.await.unwrap().unwrap();
            let all = dev_file::read(&path).unwrap();
            assert!(!all.contains_key("cloud"));
            assert_eq!(all.get("ai").map(String::as_str), Some("keep-this-key"));
        });
    }
}
