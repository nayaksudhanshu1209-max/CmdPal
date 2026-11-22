//! Shared scanner control + status (set once by the feature, used by the UI/handlers).
use super::store::Records;
use std::sync::{Arc, Condvar, Mutex, OnceLock, RwLock};

#[derive(Default)]
pub struct Control {
    pub paused: bool,
    pub rescan: bool,
    pub full: bool,
    pub clear: bool,
    pub stop: bool,
}

#[derive(Default, Clone)]
pub struct Status {
    pub state: String,
    pub entries: usize,
    pub last_scan: Option<u64>,
    pub errors: Vec<String>,
}

pub struct Shared {
    pub control: Mutex<Control>,
    pub cv: Condvar,
    pub status: Mutex<Status>,
    pub records: RwLock<Records>,
}

static SHARED: OnceLock<Arc<Shared>> = OnceLock::new();

pub fn init() -> Arc<Shared> {
    SHARED
        .get_or_init(|| {
            Arc::new(Shared {
                control: Mutex::new(Control::default()),
                cv: Condvar::new(),
                status: Mutex::new(Status { state: "Idle".into(), ..Default::default() }),
                records: RwLock::new(Records::new()),
            })
        })
        .clone()
}

fn with<R>(f: impl FnOnce(&mut Control) -> R) -> Option<R> {
    let s = SHARED.get()?;
    let mut g = s.control.lock().ok()?;
    let r = f(&mut g);
    s.cv.notify_all();
    Some(r)
}

pub fn rebuild() {
    with(|c| {
        c.rescan = true;
        c.full = true;
        c.paused = false;
    });
}

pub fn rescan() {
    with(|c| c.rescan = true);
}

pub fn toggle_pause() -> bool {
    with(|c| {
        c.paused = !c.paused;
        c.paused
    })
    .unwrap_or(false)
}

pub fn clear() {
    with(|c| c.clear = true);
}

pub fn stop() {
    with(|c| c.stop = true);
}

fn fmt_time(secs: u64) -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let ago = now.saturating_sub(secs);
    if ago < 90 {
        "just now".into()
    } else if ago < 5400 {
        format!("{} min ago", ago / 60)
    } else {
        format!("{} h ago", ago / 3600)
    }
}

pub fn status_text() -> String {
    let Some(s) = SHARED.get() else { return "File search is not running".into() };
    let st = s.status.lock().map(|g| g.clone()).unwrap_or_default();
    let mut t = format!("State: {} | Entries: {} | Last scan: {}", st.state, st.entries, st.last_scan.map(fmt_time).unwrap_or_else(|| "never".into()));
    if !st.errors.is_empty() {
        t.push_str(&format!(" | {} problem(s), latest: {}", st.errors.len(), st.errors.last().cloned().unwrap_or_default()));
    }
    t
}
