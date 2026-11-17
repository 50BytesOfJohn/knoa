//! Native messaging framing: a 32-bit length in native byte order, then the message.

use std::io::{self, Read, Write};

use anyhow::{Context as _, bail};

/// Browsers send up to 64 MiB. The largest request is a save with a page snapshot of at
/// most `MAX_SNAPSHOT_BYTES` (1 MiB), which JSON escaping can grow a few times over.
const MAX_MESSAGE: u32 = 8 * 1024 * 1024;

/// The next message's bytes, or `None` when the browser closed the pipe.
pub fn read(input: &mut impl Read) -> anyhow::Result<Option<Vec<u8>>> {
    let mut len = [0; 4];
    match input.read_exact(&mut len) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(err).context("reading the length"),
    }
    let len = u32::from_ne_bytes(len);
    if len > MAX_MESSAGE {
        bail!("message of {len} bytes is too large");
    }
    let mut body = vec![0; len as usize];
    input.read_exact(&mut body).context("reading the message")?;
    Ok(Some(body))
}

pub fn write(output: &mut impl Write, message: &serde_json::Value) -> anyhow::Result<()> {
    let body = serde_json::to_vec(message)?;
    let len = u32::try_from(body.len()).context("reply too large")?;
    output.write_all(&len.to_ne_bytes())?;
    output.write_all(&body)?;
    output.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_round_trip_until_the_pipe_closes() {
        let message = serde_json::json!({ "type": "save", "url": "https://example.com" });
        let mut framed = Vec::new();
        write(&mut framed, &message).unwrap();
        let mut input = framed.as_slice();
        let body = read(&mut input).unwrap().expect("a message");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            message
        );
        assert_eq!(read(&mut input).unwrap(), None);
    }
}
