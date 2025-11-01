use std::{
    sync::{Arc, Condvar, Mutex},
    task::Wake,
};

pub struct Parker {
    woken: Mutex<bool>,
    condvar: Condvar,
}

impl Parker {
    pub fn new() -> Self {
        Parker {
            woken: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    pub fn park(&self) {
        let mut woken = self.woken.lock().unwrap();
        while !*woken {
            woken = self.condvar.wait(woken).unwrap();
        }
        *woken = false;
    }

    fn unpark(&self) {
        let mut woken = self.woken.lock().unwrap();
        *woken = true;
        self.condvar.notify_one();
    }
}

impl Wake for Parker {
    fn wake(self: Arc<Self>) {
        self.unpark();
    }
}
