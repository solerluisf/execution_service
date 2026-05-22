use std::sync::Mutex;
use crate::core::domain::operation_mode::OperationMode;
use crate::core::infrastructure::mutex_ext::MutexExt;

pub struct ModeController {
    mode: Mutex<OperationMode>,
}

impl ModeController {
    pub fn new(initial_mode: OperationMode) -> Self {
        Self {
            mode: Mutex::new(initial_mode),
        }
    }

    pub fn get(&self) -> OperationMode {
        *self.mode.safe_lock()
    }

    pub fn set(&self, mode: OperationMode) {
        let mut m = self.mode.safe_lock();
        *m = mode;
        tracing::info!("Operation mode changed to {:?}", mode);
    }
}
