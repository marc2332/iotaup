use std::io::{self, Read, Write};
use std::time::Instant;

/// Wraps a reader and renders a progress bar to stderr as bytes flow through.
pub struct ProgressReader<R: Read> {
    inner: R,
    total: Option<u64>,
    read: u64,
    last_render: Instant,
    started: Instant,
}

impl<R: Read> ProgressReader<R> {
    pub fn new(inner: R, total: Option<u64>) -> Self {
        let now = Instant::now();
        Self {
            inner,
            total,
            read: 0,
            last_render: now - std::time::Duration::from_secs(1),
            started: now,
        }
    }

    fn render(&self, force: bool) {
        let mut out = io::stderr().lock();
        let done = self.read;
        let elapsed = self.started.elapsed().as_secs_f64().max(0.001);
        let rate = done as f64 / elapsed;
        match self.total {
            Some(total) if total > 0 => {
                let ratio = (done as f64 / total as f64).min(1.0);
                let width = 30usize;
                let filled = (ratio * width as f64) as usize;
                let bar: String = "=".repeat(filled) + &" ".repeat(width - filled);
                let _ = write!(
                    out,
                    "\r  [{bar}] {:>6.1}%  {}/{}  {}/s   ",
                    ratio * 100.0,
                    human(done),
                    human(total),
                    human(rate as u64),
                );
            }
            _ => {
                let _ = write!(out, "\r  {}  {}/s   ", human(done), human(rate as u64));
            }
        }
        let _ = out.flush();
        if force {
            let _ = writeln!(out);
        }
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self.inner.read(buf)?;
        self.read += bytes as u64;
        if bytes == 0 || self.last_render.elapsed().as_millis() >= 100 {
            self.render(false);
            self.last_render = Instant::now();
        }
        if bytes == 0 {
            self.render(true);
        }
        Ok(bytes)
    }
}

fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit_idx = 0;
    while value >= 1024.0 && unit_idx < UNITS.len() - 1 {
        value /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{bytes} {}", UNITS[unit_idx])
    } else {
        format!("{value:.1} {}", UNITS[unit_idx])
    }
}
