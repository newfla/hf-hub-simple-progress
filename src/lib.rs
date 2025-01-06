use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[cfg(feature = "sync")]
pub mod sync;

#[cfg(feature = "async")]
pub mod tokio;

/// The download progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    /// The resource to download
    pub url: String,

    /// The progress expressed as a value between 0 and 1
    pub percentage: f32,

    /// Time elapsed since the download as being started
    pub elapsed_time: Duration,

    /// Estimated time to complete the download
    pub remaining_time: Duration,
}

/// Store the state of a download
struct DownloadState {
    start_time: Instant,
    len: usize,
    offset: usize,
    url: String,
}

impl DownloadState {
    fn new(len: usize, url: &str) -> DownloadState {
        DownloadState {
            start_time: Instant::now(),
            len,
            offset: 0,
            url: url.to_string(),
        }
    }

    fn update(&mut self, delta: usize) -> Option<ProgressEvent> {
        if delta == 0 {
            return None;
        }

        self.offset += delta;

        let elapsed_time = Instant::now() - self.start_time;

        let progress = self.offset as f32 / self.len as f32;
        let progress_100 = progress * 100.;

        let remaining_percentage = 100. - progress_100;
        let duration_unit = elapsed_time / progress_100 as u32;
        let remaining_time = duration_unit * remaining_percentage as u32;

        let event = ProgressEvent {
            url: self.url.clone(),
            percentage: progress,
            elapsed_time,
            remaining_time,
        };
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use crate::DownloadState;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn it_works() {
        let mut state = DownloadState::new(10, "https://www.rust-lang.org");

        assert!(state.update(0).is_none());
        sleep(Duration::from_secs(1));

        let mid_update = state.update(5).unwrap();
        assert_eq!(0.5, mid_update.percentage);
        assert!(mid_update.elapsed_time.as_secs_f32() > 1.);
        assert!(mid_update.remaining_time.as_secs_f32() > 1.);

        let end_update = state.update(5).unwrap();
        assert_eq!(1., end_update.percentage);
        assert_eq!(0., end_update.remaining_time.as_secs_f32());
    }
}
