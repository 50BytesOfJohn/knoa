//! What the search pill holds: tokens the user has locked in, and the text still being
//! typed. Together with the place they make one `Query`.

use knoa_core::{Filter, Kind, Place, Query, WorkspaceId};

use crate::i18n;

/// One locked-in part of the search.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Filter(Filter),
    /// A word or phrase searched as text, drawn as its own chip.
    Text(String),
}

impl Token {
    /// The token as it would be typed: `#pasta`, `kind:recipe`, `quick`.
    pub fn typed(&self) -> String {
        match self {
            Token::Filter(filter) => filter.typed(),
            Token::Text(text) => text.clone(),
        }
    }
}

/// The search apart from the place. Text tokens stay apart from the typed text so the
/// pill can draw them; both are searched as text.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Search {
    pub tokens: Vec<Token>,
    /// What is in the input, as typed: kind names in the user's language.
    pub typed: String,
}

impl Search {
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.typed.trim().is_empty()
    }

    /// Whether any text is searched, which ranks results by best match.
    pub fn has_text(&self) -> bool {
        !self.typed.trim().is_empty() || self.tokens.iter().any(|t| matches!(t, Token::Text(_)))
    }

    pub fn kind(&self) -> Option<Kind> {
        self.tokens.iter().find_map(|token| match token {
            Token::Filter(Filter::Kind(kind)) => Some(*kind),
            _ => None,
        })
    }

    pub fn contains(&self, filter: &Filter) -> bool {
        self.tokens
            .iter()
            .any(|t| matches!(t, Token::Filter(f) if f == filter))
    }

    /// Adds a token at the end. A kind replaces the kind token already there, in place,
    /// since only one kind applies; a token already there isn't added twice.
    pub fn add(&mut self, token: Token) {
        if self.tokens.contains(&token) {
            return;
        }
        let is_kind = |t: &Token| matches!(t, Token::Filter(Filter::Kind(_)));
        match self.tokens.iter_mut().find(|t| is_kind(t)) {
            Some(existing) if is_kind(&token) => *existing = token,
            _ => self.tokens.push(token),
        }
    }

    /// Everything the search finds within `place`.
    pub fn query(&self, workspace: WorkspaceId, place: Place) -> Query {
        let mut query = self.base_query(workspace, place);
        let text: Vec<&str> = query
            .text
            .split_whitespace()
            .chain(self.typed.split_whitespace())
            .collect();
        query.text = text.join(" ");
        query
    }

    /// The query without the typed text: what suggestions count within, so each one
    /// narrows the view the tokens already make.
    pub fn base_query(&self, workspace: WorkspaceId, place: Place) -> Query {
        let mut query = Query {
            workspace: Some(workspace),
            place,
            ..Query::default()
        };
        let mut text = Vec::new();
        for token in &self.tokens {
            match token {
                Token::Filter(filter) => query.apply(filter.clone()),
                Token::Text(words) => text.push(words.as_str()),
            }
        }
        query.text = text.join(" ");
        query
    }

    /// The search as one line of syntax, `kind:recipe #pasta quick`, which parses back to
    /// the same tokens. The typed text is left as typed.
    pub fn typed_form(&self) -> String {
        let tokens = self.tokens.iter().map(Token::typed);
        let typed = (!self.typed.trim().is_empty()).then(|| self.typed.trim().to_owned());
        tokens.chain(typed).collect::<Vec<_>>().join(" ")
    }

    /// Takes what the input holds. Every complete word (one followed by whitespace) with
    /// known syntax becomes a token and leaves the text; this covers typing a space after
    /// `#pasta` and pasting `kind:recipe #pasta quick`. Returns whether any word did, in
    /// which case the input has to show the shorter text.
    pub fn take_input(&mut self, input: &str) -> bool {
        let complete_end = input.rfind(char::is_whitespace).map_or(0, |i| {
            i + input[i..].chars().next().map_or(1, char::len_utf8)
        });
        let (complete, tail) = input.split_at(complete_end);
        let mut kept = String::new();
        let mut took = false;
        for word in complete.split_whitespace() {
            if let Some(filter) = Filter::parse(&i18n::with_kind_slugs(word)) {
                self.add(Token::Filter(filter));
                took = true;
            } else {
                kept.push_str(word);
                kept.push(' ');
            }
        }
        self.typed = if took { kept + tail } else { input.to_owned() };
        took
    }
}
