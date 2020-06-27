use std::thread::{self, JoinHandle};
use std::sync::{Mutex, Arc, Condvar};
use std::time::{Instant, Duration};
use take_mut;
use rppal::gpio::*;

/// Enables a digital input to be wrapped into a debounced input.
pub trait Debounce<T> where T: Debounced {
    fn debounce(self, time: Duration) -> T;
}

/// Represents a debounced digital input.
pub trait Debounced {
    fn on_changed<C>(&mut self, callback: C) 
        where C: FnMut(bool) + Send + 'static;

    fn is_high(&self) -> bool;

    fn is_low(&self) -> bool;

    fn set_bounce_time(&mut self, time: Duration);
}

/// Simple wrapper around `rppal::gpio::pin::InputPin` to add debouncing.
pub struct SoftInputPin {
    pin: InputPin,
    state: Arc<Mutex<SoftInputState>>,
    pin_status: Arc<(Mutex<bool>, Condvar)>,
    handler_thread: Option<JoinHandle<()>>,
}

struct SoftInputState {
    bounce_time: Duration,
    last_changed: Instant,
    last_value: bool,
    change_callback: Option<Box<dyn FnMut(bool) + Send + 'static>>
}

impl SoftInputState {
    fn change_last_value(&mut self, new_value: bool) {
        self.last_changed = Instant::now();
        self.last_value = new_value;
    }
}

impl SoftInputPin {
    fn new(mut pin: InputPin, bounce_time: Duration) -> Self {
        pin.set_interrupt(Trigger::Both).unwrap();
        let last_changed = Instant::now();
        let last_value = pin.is_high();
        let pin_status = Arc::new((Mutex::new(last_value), Condvar::new()));

        let state = SoftInputState {
            last_changed,
            bounce_time,
            last_value,
            change_callback: None
        };

        let mut s = Self {
            pin,
            state: Arc::new(Mutex::new(state)),
            pin_status,
            handler_thread: None
        };
        s.start_handler_thread();
        s
    }

    fn start_handler_thread(&mut self) {
        let pin_status = Arc::clone(&self.pin_status);
        let state = Arc::clone(&self.state);

        self.handler_thread = Some(thread::spawn(move || {
            let (status_mutex, cvar) = &*pin_status;
            let mut status_lock = status_mutex.lock().unwrap();
            loop {
                // Wait for notification from interrupt
                status_lock = cvar.wait(status_lock).unwrap();

                let new_value = *status_lock;

                // Acquire debounce state and pin
                let mut state = state.lock().unwrap();
                
                // Ignore this event if the state hasn't changed
                if new_value != state.last_value {

                    // Inform user of value change
                    state.change_last_value(new_value);
                    if let Some(callback) = state.change_callback.as_mut() {
                        callback(new_value);
                    }

                    // Sleep for bounce time, but allow input interrupts to acquire lock on status_mutex
                    take_mut::take(&mut status_lock, |status_lock| {
                        drop(status_lock);
                        thread::sleep(state.bounce_time);
                        status_mutex.lock().unwrap()
                    });

                    // Check if value has changed; if so, notify user again
                    let next_value = *status_lock;
                    if next_value != state.last_value {
                        state.change_last_value(next_value);
                        if let Some(callback) = state.change_callback.as_mut() {
                            callback(next_value);
                        }
                    }
                }
            }
        }));

        let pin_status = Arc::clone(&self.pin_status);
        self.pin.set_async_interrupt(Trigger::Both, move |level| {
            let (status_mutex, cvar) = &*pin_status;
            let mut status_lock = status_mutex.lock().unwrap();
            *status_lock = level == Level::High;
            cvar.notify_one();
        }).unwrap();
    }
}

impl Debounce<SoftInputPin> for InputPin {
    fn debounce(self, time: Duration) -> SoftInputPin {
        SoftInputPin::new(self, time)
    }
}

impl Debounced for SoftInputPin {
    fn on_changed<C>(&mut self, mut callback: C)
    where C: FnMut(bool) + Send + 'static {
        let mut state = self.state.lock().unwrap();
        state.change_callback = Some(Box::new(callback));
    }

    fn is_high(&self) -> bool {
        self.state.lock().unwrap().last_value
    }

    #[inline]
    fn is_low(&self) -> bool {
        !self.is_high()
    }

    fn set_bounce_time(&mut self, time: Duration) {
        let mut state = self.state.lock().unwrap();
        state.bounce_time = time;
    }
}