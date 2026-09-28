use std::io::SeekFrom;
use std::time::Duration;

use libmpv2::protocol::Protocol;
use libmpv2::Mpv;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::torrent::TorrentHandle;

const READ_TIMEOUT: Duration = Duration::from_secs(20);
const OPEN_TIMEOUT: Duration = Duration::from_secs(20);

trait ReadSeek: tokio::io::AsyncRead + tokio::io::AsyncSeek + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncSeek + Unpin + Send> ReadSeek for T {}

pub struct TorrentStreamCookie {
    stream: Box<dyn ReadSeek>,
    len: u64,
    rt: tokio::runtime::Handle,
}

impl std::panic::RefUnwindSafe for TorrentStreamCookie {}

/// Registers the `torrent://` protocol on this mpv instance. Registering
/// the same name twice returns an InvalidParameter error rather than
/// panicking, so a double call fails loudly but safely.
pub fn register(mpv: &Mpv, torrent: TorrentHandle) -> libmpv2::Result<()> {
    let protocol = unsafe {
        Protocol::new(mpv, "torrent".into(), torrent, open, close, read, Some(seek), Some(size))
    };
    protocol.register()?;

    // mpv keeps a raw pointer into `protocol`'s data for as long as it runs.
    // Dropping it here would free memory mpv still uses. It lives for the app's
    // lifetime anyway (one Mpv, one registration), so leaking it is correct.
    std::mem::forget(protocol);
    Ok(())
}

fn open(handle: &mut TorrentHandle, uri: &str) -> TorrentStreamCookie {
    let session_id = uri
        .strip_prefix("torrent://")
        .expect("registered for torrent:// only");

    let (torrent, file_index, len, rt) = handle
        .lookup_session(session_id)
        .expect("unknown or expired torrent stream session");

    let stream = rt
        .block_on(async {
            tokio::time::timeout(OPEN_TIMEOUT, torrent.stream(file_index)).await
        })
        .expect("timed out opening torrent file stream")
        .expect("failed to open torrent file stream");

    TorrentStreamCookie { stream: Box::new(stream), len, rt }
}

fn read(cookie: &mut TorrentStreamCookie, buf: &mut [i8]) -> i64 {
    let buf_u8: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len()) };

    let rt = cookie.rt.clone();
    let result = rt.block_on(async {
        tokio::time::timeout(READ_TIMEOUT, cookie.stream.read(buf_u8)).await
    });

    match result {
        Ok(Ok(n)) => n as i64,
        _ => -1, // timeout (dead swarm) or read error
    }
}

fn seek(cookie: &mut TorrentStreamCookie, offset: i64) -> i64 {
    let rt = cookie.rt.clone();
    match rt.block_on(async { cookie.stream.seek(SeekFrom::Start(offset as u64)).await }) {
        Ok(pos) => pos as i64,
        Err(_) => -1,
    }
}
fn close(_cookie: Box<TorrentStreamCookie>) {
    // Session teardown is driven by the watch page's navigation-away hook,
    // not here. mpv can reopen the same URI on internal retries.
}

fn size(cookie: &mut TorrentStreamCookie) -> i64 {
    cookie.len as i64
}