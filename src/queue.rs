use const_ft::const_ft;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m::interrupt::CriticalSection;

/// A queue of fixed length based around a mutable slice provided to the
/// constructor. Holds a number of some type `T`. Safe for multiple consumers
/// and producers.
pub struct AtomicQueue<'a, T>
where
    T: 'a + Copy,
{
    /// This is where we store our data
    data: UnsafeCell<&'a mut [T]>,
    /// This is the counter for the first item in the queue (i.e. the one to
    /// be pop'd next). Counters increment forever. You convert it to an
    /// array index by taking them modulo data.len() (see `counter_to_idx`).
    read: AtomicUsize,
    /// This is the counter for the last item in the queue (i.e. the most
    /// recent one pushed), whether or not the write is complete. Counters
    /// increment forever. You convert it to an array index by taking them
    /// modulo data.len() (see `counter_to_idx`).
    write: AtomicUsize,
    /// This is the counter for the last item in the queue (i.e. the most
    /// recent one pushed) where the write is actually complete. Counters
    /// increment forever. You convert it to an array index by taking them
    /// modulo data.len() (see `counter_to_idx`).
    available: AtomicUsize,
}

/// Our use of CAS atomics means we can share `AtomicQueue` between threads
/// safely.
unsafe impl<'a, T> Sync for AtomicQueue<'a, T> where T: Send + Copy {}

impl<'a, T> AtomicQueue<'a, T>
where
    T: Copy,
{
    /// Create a new queue.
    ///
    /// buffer is a mutable slice which this queue will use as storage. It
    /// needs to live at least as long as the queue does.
    pub fn new() -> AtomicQueue<'a, T> {
        let mut buf: [MaybeUninit<T>; 32] =
            unsafe { MaybeUninit::<[MaybeUninit<T>; 32]>::uninit().assume_init() };
        AtomicQueue {
            data: UnsafeCell::new(unsafe { &mut *(&mut buf as *mut [MaybeUninit<T>] as *mut [T]) }),
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
            available: AtomicUsize::new(0),
        }
    }

    /// Return the length of the queue. Note, we do not 'reserve' any
    /// elements, so you can actually put `N` items in a queue of length `N`.
    pub fn length(&self) -> usize {
        unsafe { (*self.data.get()).len() }
    }

    /// Add an item to the queue. An error is returned if the queue is full.
    /// You can call this from an ISR but don't call it from two different ISR
    /// concurrently. You can call it from two different threads just fine
    /// though, as long as they're pre-emptible
    pub fn push(&self, value: T, _cs: &CriticalSection) -> Result<(), ()> {
        // Loop until we've allocated ourselves some space without colliding
        // with another writer.
        let read = self.read.load(Ordering::Acquire);
        let write = self.write.load(Ordering::Relaxed);
        if (write.wrapping_sub(read)) >= self.length() {
            // Queue is full - quit now
            return Err(());
        }

        self.write.store(write.wrapping_add(1), Ordering::Relaxed);

        // This is safe because we're the only possible thread with this value
        // of idx (reading or writing).
        let p = unsafe { &mut *self.data.get() };
        p[self.counter_to_idx(write)] = value;

        // Now update `self.available` so that readers can read what we just wrote.

        self.available
            .store(write.wrapping_add(1), Ordering::Release);
        return Ok(());
    }

    /// Take the next item off the queue. `None` is returned if the queue is
    /// empty. You can call this from multiple ISRs or threads concurrently
    /// and it should be fine.
    pub fn pop(&self, _cs: &CriticalSection) -> Option<T> {
        let available = self.available.load(Ordering::Acquire);
        let read = self.read.load(Ordering::Relaxed);
        if read >= available {
            // Queue is empty - quit now
            return None;
        }

        // This is safe because no-one else can be writing to this
        // location, and concurrent reads get resolved in the next block.
        let p = unsafe { &*self.data.get() };
        // Cache the result
        let result = p[self.counter_to_idx(read)];

        self.read.store(read.wrapping_add(1), Ordering::Release);
        return Some(result);
    }

    /// Counters are converted to slice indexes by taking them modulo the
    /// slice length.
    fn counter_to_idx(&self, counter: usize) -> usize {
        counter % self.length()
    }
}
