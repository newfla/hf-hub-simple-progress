use crate::{DownloadState, ProgressEvent};
use hf_hub::api::tokio::Progress;
use std::ops::AsyncFnMut;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct CallbackStorage<C> {
    download_state: Arc<Mutex<Option<DownloadState>>>,
    callback: C,
}

impl<C> Progress for CallbackStorage<C>
where
    C: AsyncFnMut(ProgressEvent) -> () + Send + Sync + Clone,
    for<'a> C::CallRefFuture<'a>: Send + Sync,
{
    async fn init(&mut self, size: usize, filename: &str) {
        self.download_state = Arc::new(Mutex::new(Some(DownloadState::new(size, filename))));
    }

    async fn update(&mut self, size: usize) {
        let update = self
            .download_state
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .update(size);
        if let Some(delta) = update {
            (self.callback)(delta).await;
        }
    }

    async fn finish(&mut self) {
        // Nothing to do
    }
}

/// Build a [hf_hub::api::Progress] that encapsulate the provided callback
pub fn callback_builder<C>(callback: C) -> impl Progress + Clone
where
    C: AsyncFnMut(ProgressEvent) + Clone + Send + Sync + 'static,
    for<'a> C::CallRefFuture<'a>: Send + Sync,
{
    CallbackStorage {
        download_state: Default::default(),
        callback: Box::new(callback),
    }
}

#[cfg(test)]
mod tests {
    use crate::async_closure::callback_builder;
    use crate::ProgressEvent;
    use hf_hub::api::tokio::ApiBuilder;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    #[tokio::test]
    async fn it_works() {
        let done = Arc::new(AtomicBool::new(false));
        let done_copy = done.clone();
        let api = ApiBuilder::new().build().unwrap();
        let callback = callback_builder(async move |progress: ProgressEvent| {
            if progress.percentage == 1. {
                done_copy.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        });
        api.model("ggerganov/whisper.cpp".to_string())
            .download_with_progress("ggml-tiny.en-q5_1.bin", callback)
            .await
            .unwrap();
        assert!(done.load(std::sync::atomic::Ordering::Relaxed));
    }
}
