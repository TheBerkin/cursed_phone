use std::thread::{self, JoinHandle};
use std::sync::{Mutex, Arc, atomic::{Ordering, AtomicBool}, mpsc};
use std::time::{Instant, Duration};
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

    fn is_debouncing(&self) -> bool;

    fn set_bounce_time(&mut self, time: Duration);
}

/// Simple wrapper around `rppal::gpio::pin::InputPin` to add debouncing.
pub struct SoftInputPin {
    pin: InputPin,
    state: Arc<Mutex<SoftInputState>>,
    debounce_flag: Arc<AtomicBool>,
    tx_handler: mpsc::Sender<bool>,
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

        let (tx_handler, rx) = mpsc::channel();

        let state = SoftInputState {
            last_changed,
            bounce_time,
            last_value,
            change_callback: None
        };

        let mut s = Self {
            pin,
            tx_handler,
            state: Arc::new(Mutex::new(state)),
            debounce_flag: Arc::new(AtomicBool::new(false)),
            handler_thread: None
        };
        s.start_handler_thread(rx);
        s        
    }

    fn start_handler_thread(&mut self, rx: mpsc::Receiver<bool>) {
        let debounce_flag = Arc::clone(&self.debounce_flag);
        let state = Arc::clone(&self.state);

        self.handler_thread = Some(thread::spawn(move || {
            while let Ok(new_value) = rx.recv() {
                // If the pin is currently debouncing, ignore this event entirely.
                if debounce_flag.load(Ordering::SeqCst) { return }

                // Acquire debounce state and pin
                let mut state = state.lock().unwrap();
                
                // Ignore this event if the state hasn't changed
                if new_value != state.last_value {
                    // Enable debounce flag
                    debounce_flag.store(true, Ordering::SeqCst);

                    // Inform user of value change
                    state.change_last_value(new_value);
                    if let Some(callback) = state.change_callback.as_mut() {
                        callback(new_value);
                    }

                    // Sleep for bounce time
                    thread::sleep(state.bounce_time);

                    // Check if input has changed since debounce and inform user if so
                    let next_value = match rx.try_recv() {
                        Ok(is_high) => is_high,
                        _ => new_value
                    };

                    if next_value != state.last_value {
                        state.change_last_value(next_value);
                        if let Some(callback) = state.change_callback.as_mut() {
                            callback(next_value);
                        }
                    }

                    // End debounce
                    debounce_flag.store(false, Ordering::SeqCst);
                }
            }
        }));

        let debounce_flag2 = Arc::clone(&self.debounce_flag);
        let tx_handler = self.tx_handler.clone();
        self.pin.set_async_interrupt(Trigger::Both, move |level| {
            if debounce_flag2.load(Ordering::SeqCst) { return }
            tx_handler.send(level == Level::High).unwrap();
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

    #[inline]
    fn is_debouncing(&self) -> bool {
        self.debounce_flag.load(Ordering::SeqCst)
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