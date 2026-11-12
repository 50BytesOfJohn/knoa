//! Replaces a library's account link only while the requesting session still owns it.

use gpui::{AppContext as _, Context};

use super::{Cloud, Phase};

impl Cloud {
    /// The user agreed to replace the other account's link: this library becomes the
    /// signed-in account's, merged with what it already has.
    pub fn replace_other_account(&mut self, cx: &mut Context<'_, Cloud>) {
        let Phase::OtherAccount(connection) = &mut self.phase else {
            return;
        };
        if connection.replacement.is_some() {
            return;
        }
        let api = connection.api.clone();
        let account = connection.account.clone();
        let database = self.library.read(cx).database();
        let generation = self.generation.ticket();
        connection.replacement = Some(cx.spawn(async move |this, cx| {
            let before_unlink = generation.clone();
            let unlinked = cx
                .background_spawn(
                    async move { before_unlink.if_current(|| database.write().unlink()) },
                )
                .await;
            let _ = this.update(cx, |this, cx| {
                if generation.if_current(|| ()).is_none() {
                    return;
                }
                match unlinked {
                    Some(Ok(())) => this.start(api, account, cx),
                    Some(Err(err)) => {
                        if let Phase::OtherAccount(connection) = &mut this.phase {
                            connection.replacement = None;
                        }
                        this.problem = Some(format!("Couldn't unlink the library: {err}"));
                        cx.notify();
                    }
                    None => {}
                }
            });
        }));
    }
}

#[cfg(test)]
mod tests {
    use crate::services::generation::Generation;

    #[test]
    fn a_superseded_session_cannot_unlink_or_restart() {
        let session = Generation::default();
        let before_unlink = session.ticket();
        session.advance();
        let mut unlinked = false;
        assert_eq!(before_unlink.if_current(|| unlinked = true), None);
        assert!(!unlinked);

        let before_restart = session.ticket();
        assert_eq!(before_restart.if_current(|| unlinked = true), Some(()));
        assert!(unlinked);
        session.advance();
        let mut restarted = false;
        assert_eq!(before_restart.if_current(|| restarted = true), None);
        assert!(!restarted);
        assert_eq!(
            session.ticket().if_current(|| "new session"),
            Some("new session")
        );
    }
}
