use std::time::{Duration, Instant};

use licht_core::DownloadProgress;

/// Emits at most every 100 ms, and always when a phase finishes or the total changes.
pub struct ProgressThrottle {
    previous_total: Option<u32>,
    last_emit: Option<Instant>,
}

impl ProgressThrottle {
    pub fn new() -> Self {
        Self {
            previous_total: None,
            last_emit: None,
        }
    }

    pub fn should_emit(&mut self, progress: DownloadProgress, now: Instant) -> bool {
        let complete = progress.total > 0 && progress.finished + progress.failed == progress.total;
        let phase_changed = self
            .previous_total
            .is_some_and(|total| total != progress.total);
        let due = self
            .last_emit
            .is_none_or(|drawn| now.saturating_duration_since(drawn) >= Duration::from_millis(100));
        if !complete && !phase_changed && !due {
            return false;
        }
        self.previous_total = Some(progress.total);
        self.last_emit = Some(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use licht_core::DownloadProgress;

    use super::ProgressThrottle;

    fn progress(finished: u32, total: u32) -> DownloadProgress {
        DownloadProgress {
            finished,
            failed: 0,
            total,
            bytes_done: u64::from(finished),
            bytes_total: u64::from(total),
        }
    }

    #[test]
    fn a_burst_inside_the_window_emits_once_unless_the_phase_ends() {
        let start = Instant::now();
        let mut gate = ProgressThrottle::new();
        assert!(gate.should_emit(progress(1, 10), start));
        assert!(!gate.should_emit(progress(2, 10), start + Duration::from_millis(40)));
        assert!(gate.should_emit(progress(10, 10), start + Duration::from_millis(50)));
        assert!(gate.should_emit(progress(0, 4), start + Duration::from_millis(60)));
        assert!(gate.should_emit(progress(1, 4), start + Duration::from_millis(160)));
    }
}
