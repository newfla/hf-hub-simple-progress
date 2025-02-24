use crate::{DownloadState, ProgressEvent};
use hf_hub::api::Progress;

struct CallbackStorage<C> {
    download_state: Option<DownloadState>,
    callback: C,
}

impl<C> Progress for CallbackStorage<C>
where
    C: FnMut(ProgressEvent),
{
    fn init(&mut self, size: usize, filename: &str) {
        self.download_state = Some(DownloadState::new(size, filename));
    }

    fn update(&mut self, size: usize) {
        if let Some(delta) = self.download_state.as_mut().unwrap().update(size) {
            (self.callback)(delta);
        }
    }

    fn finish(&mut self) {
        // Nothing to do
    }
}

/// Build a [hf_hub::api::Progress] that encapsulate the provided callback
pub fn callback_builder(callback: impl FnMut(ProgressEvent) + 'static) -> impl Progress {
    CallbackStorage {
        download_state: None,
        callback: Box::new(callback),
    }
}

#[cfg(test)]
mod tests {
    use crate::ProgressEvent;
    use crate::sync::callback_builder;
    use hf_hub::api::sync::ApiBuilder;
    use std::rc::Rc;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn it_works() {
        let done = Rc::new(AtomicBool::new(false));
        let done_copy = done.clone();
        let api = ApiBuilder::new().build().unwrap();
        let callback = callback_builder(move |progress: ProgressEvent| {
            if progress.percentage == 1. {
                done_copy.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        });
        api.model("ggerganov/whisper.cpp".to_string())
            .download_with_progress("ggml-tiny-q5_1.bin", callback)
            .unwrap();
        assert!(done.load(std::sync::atomic::Ordering::Relaxed));
    }
}
