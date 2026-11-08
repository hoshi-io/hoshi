use std::ffi::c_char;
use std::io::SeekFrom;
use std::time::Duration;

use libmpv2::protocol::{Protocol};
use libmpv2::Mpv;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tracing::warn;

use crate::torrent::TorrentHandle;

const READ_TIMEOUT: Duration = Duration::from_secs(20);
const OPEN_TIMEOUT: Duration = Duration::from_secs(20);

trait ReadSeek: tokio::io::AsyncRead + tokio::io::AsyncSeek + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncSeek + Unpin + Send> ReadSeek for T {}

/// `stream` is `None` when opening failed. libmpv2's open callback can only
/// signal failure by panicking, and with `panic = "abort"` in release that
/// would kill the process, so we hand back a dead cookie instead and fail
/// every read/seek, which mpv reports as a normal load error.
pub struct TorrentStreamCookie {
    stream: Option<Box<dyn ReadSeek>>,
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
    open_with(handle, uri, "torrent://")
}

fn open_thumb(handle: &mut TorrentHandle, uri: &str) -> TorrentStreamCookie {
    open_with(handle, uri, "torrentthumb://")
}


fn read_thumb(cookie: &mut TorrentStreamCookie, buf: &mut [c_char]) -> i64 {
    // data not there in time: no thumbnail
    read_with_timeout(cookie, buf, THUMB_READ_TIMEOUT)
}

pub fn register_thumb(mpv: &Mpv, torrent: TorrentHandle) -> libmpv2::Result<()> {
    let protocol = unsafe {
        Protocol::new(mpv, "torrentthumb".into(), torrent, open_thumb, close, read_thumb, Some(seek), Some(size))
    };
    protocol.register()?;
    std::mem::forget(protocol);
    Ok(())
}

const THUMB_READ_TIMEOUT: Duration = Duration::from_millis(1500);

fn open_with(handle: &mut TorrentHandle, uri: &str, prefix: &str) -> TorrentStreamCookie {
    let rt = handle.runtime();
    let dead = |rt: tokio::runtime::Handle| TorrentStreamCookie { stream: None, len: 0, rt };

    let Some(session_id) = uri.strip_prefix(prefix) else {
        warn!(uri, "torrent protocol: unexpected uri scheme");
        return dead(rt);
    };
    let Some((torrent, file_index, len, rt)) = handle.lookup_session(session_id) else {
        warn!(session_id, "torrent protocol: unknown or expired stream session");
        return dead(rt);
    };
    match rt.block_on(async { tokio::time::timeout(OPEN_TIMEOUT, torrent.stream(file_index)).await }) {
        Ok(Ok(stream)) => TorrentStreamCookie { stream: Some(Box::new(stream)), len, rt },
        Ok(Err(e)) => {
            warn!(session_id, error = ?e, "torrent protocol: failed to open file stream");
            dead(rt)
        }
        Err(_) => {
            warn!(session_id, "torrent protocol: timed out opening file stream");
            dead(rt)
        }
    }
}

fn read_with_timeout(cookie: &mut TorrentStreamCookie, buf: &mut [c_char], timeout: Duration) -> i64 {
    let Some(stream) = cookie.stream.as_mut() else { return -1 };
    let buf_u8 = unsafe {
        std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len())
    };
    match cookie.rt.block_on(async { tokio::time::timeout(timeout, stream.read(buf_u8)).await }) {
        Ok(Ok(n)) => n as i64,
        _ => -1,
    }
}

fn read(cookie: &mut TorrentStreamCookie, buf: &mut [c_char]) -> i64 {
    read_with_timeout(cookie, buf, READ_TIMEOUT)
}

fn seek(cookie: &mut TorrentStreamCookie, offset: i64) -> i64 {
    let Some(stream) = cookie.stream.as_mut() else { return -1 };
    match cookie.rt.block_on(async { stream.seek(SeekFrom::Start(offset as u64)).await }) {
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