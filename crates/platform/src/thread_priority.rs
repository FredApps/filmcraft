//! Scheduling priority of background work (ADR 0001, amended): the threads of an export, Render
//! Previews or proxy job ([`filmcraft_export::on_own_threads`]) run at the utility
//! quality-of-service class, below the default class of the monitors' frame workers and the
//! user-interactive main thread, so editing stays responsive while an export runs. A thread a
//! utility thread creates starts at the default class again (macOS does not pass the class on),
//! so decoders an export opens, which the monitors later share, keep their priority.

/// Lower the calling thread to the utility class. A thread the system refuses stays as it was.
pub fn lower_current_thread() {
    // SAFETY: `pthread_set_qos_class_self_np` changes only the calling thread's own scheduling
    // class; it takes two plain integers and has no memory-safety preconditions.
    let r = unsafe { libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_UTILITY, 0) };
    if r != 0 {
        log::debug!("could not lower a background thread's priority (error {r})");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The calling thread's QoS class, as its raw value.
    fn class() -> u32 {
        let mut class = libc::qos_class_t::QOS_CLASS_UNSPECIFIED;
        let mut relative = 0;
        // SAFETY: `pthread_self` is the calling thread, alive for the call; both out-pointers are
        // valid, writable locals.
        let r = unsafe { libc::pthread_get_qos_class_np(libc::pthread_self(), &mut class, &mut relative) };
        assert_eq!(r, 0);
        class as u32
    }

    #[test]
    fn lowers_only_the_calling_thread() {
        std::thread::spawn(|| {
            assert_eq!(class(), libc::qos_class_t::QOS_CLASS_DEFAULT as u32);
            lower_current_thread();
            assert_eq!(class(), libc::qos_class_t::QOS_CLASS_UTILITY as u32);
            // a thread it creates is not lowered with it
            let child = std::thread::spawn(class).join().unwrap();
            assert_eq!(child, libc::qos_class_t::QOS_CLASS_DEFAULT as u32);
        })
        .join()
        .unwrap();
    }
}
