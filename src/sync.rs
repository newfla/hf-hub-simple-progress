use crate::{DownloadState, ProgressEvent};
use hf_hub::api::Progress;

pub struct CallbackStorage {
    download_state: Option<DownloadState>,
    callback: Box<dyn FnMut(ProgressEvent)>,
}

impl Progress for CallbackStorage {
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

pub fn callback_builder<C: FnMut(ProgressEvent) + 'static>(callback: C) -> impl Progress {
    let storage = CallbackStorage {
        download_state: None,
        callback: Box::new(callback),
    };
    storage
}

#[cfg(test)]
mod tests {
    use crate::sync::callback_builder;
    use crate::ProgressEvent;
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
        api.model("julien-c/dummy-unknown".to_string())
            .download_with_progress("config.json", callback)
            .unwrap();
        assert!(done.load(std::sync::atomic::Ordering::Relaxed));
    }
}
