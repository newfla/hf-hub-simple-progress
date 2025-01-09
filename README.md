# hf-hub-progress-callback

WIP 

Based on my previous work [hf-hub#70](https://github.com/huggingface/hf-hub/pull/70/)

## Usage 
### Sync
``` rust
use std::{thread::sleep, time::Duration, sync::atomic::AtomicBool, rc::Rc};

use hf_hub::api::sync::ApiBuilder;
use hf_hub_progress_callback::{ProgressEvent, sync::callback_builder};

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
```

### Async 
See the test section in the ``async_closure`` module.

Requires ``async_fn_traits, unboxed_closures`` nightly features 