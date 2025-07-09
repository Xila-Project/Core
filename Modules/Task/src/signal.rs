use core::fmt::Debug;

/// POSIX signals enumeration
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Signal_type {
    /// Hangup detected on controlling terminal or death of controlling process (SIGHUP)
    Hangup = 0,
    /// Interrupt from keyboard (SIGINT)
    Interrupt,
    /// Quit from keyboard (SIGQUIT)
    Quit,
    /// Unused : Illegal Instruction (SIGILL)
    Illegal_instruction,
    /// Trace/breakpoint trap (SIGTRAP)
    Trap,
    /// Abort signal from abort(3) (SIGABRT)
    Abort,
    /// Bus error (bad memory access) (SIGBUS)
    Bus_error,
    /// Floating-point exception (SIGFPE)
    Floating_point_exception,
    /// Kill signal (SIGKILL)
    Kill,
    /// User-defined signal 1 (SIGUSR1)
    User_1,
    /// Invalid memory reference (SIGSEGV)
    Segmentation_fault,
    /// User-defined signal 2 (SIGUSR2)
    User_2,
    /// Broken pipe: write to pipe with no readers (SIGPIPE)
    Broken_pipe,
    /// Timer signal from alarm(2) (SIGALRM)
    Alarm,
    /// Termination signal (SIGTERM)
    Termination,
    /// Stack fault on coprocessor (unused) (SIGSTKFLT)
    Stack_fault,
    /// Child stopped or terminated (SIGCHLD)
    Child,
    /// Continue if stopped (SIGCONT)
    Continue,
    /// Stop process (SIGSTOP)
    Stop,
    /// Stop typed at terminal (SIGTSTP)
    Terminal_stop,
    /// Terminal input for background process (SIGTTIN)
    Terminal_input,
    /// Terminal output for background process (SIGTTOU)
    Terminal_output,
    /// Urgent condition on socket (4.2BSD) (SIGURG)
    Urgent,
    /// CPU time limit exceeded (4.2BSD) (SIGXCPU)
    Cpu_time_limit_exceeded,
    /// File size limit exceeded (4.2BSD) (SIGXFSZ)
    File_size_limit_exceeded,
    /// Virtual alarm clock (4.2BSD) (SIGVTALRM)
    Virtual_alarm,
    /// Profiling timer expired (SIGPROF)
    Profiling_timer_expired,
    /// Window resize signal (4.3BSD, Sun) (SIGWINCH)
    Window_resize,
    /// I/O now possible (4.2BSD) (SIGIO)
    IO_Possible,
    /// Power failure (System V) (SIGPWR)
    Power_failure,
    /// Bad system call (SVr4) (SIGSYS)
    Bad_system_call,
}

impl Signal_type {
    pub const FIRST: Self = Self::Hangup;
    pub const LAST: Self = Self::Bad_system_call;

    pub const fn get_discriminant(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Signal_accumulator_type {
    accumulator: u32,
}

impl Signal_accumulator_type {
    pub const fn new() -> Self {
        Self { accumulator: 0 }
    }

    pub const fn send(&mut self, signal: Signal_type) {
        self.accumulator |= 1 << signal as u32;
    }

    pub const fn clear(&mut self, signal: Signal_type) {
        self.accumulator &= !(1 << signal as u32);
    }

    pub const fn has_signal(&self, signal: Signal_type) -> bool {
        self.accumulator & (1 << signal as u32) != 0
    }

    pub fn peek(&self) -> Option<Signal_type> {
        for bit in Signal_type::FIRST as u8..=Signal_type::LAST as u8 {
            let signal = unsafe { core::mem::transmute::<u8, Signal_type>(bit) };

            if self.has_signal(signal) {
                return Some(signal);
            }
        }

        None
    }

    pub fn pop(&mut self) -> Option<Signal_type> {
        if let Some(signal) = self.peek() {
            self.clear(signal);

            Some(signal)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_and_has_signal() {
        let mut acc = Signal_accumulator_type::new();
        acc.send(Signal_type::Interrupt);
        assert!(acc.has_signal(Signal_type::Interrupt));
    }

    #[test]
    fn test_clear_signal() {
        let mut acc = Signal_accumulator_type::new();
        acc.send(Signal_type::Quit);
        acc.clear(Signal_type::Quit);
        assert!(!acc.has_signal(Signal_type::Quit));
    }

    #[test]
    fn test_peek_and_pop() {
        let mut acc = Signal_accumulator_type::new();
        acc.send(Signal_type::Hangup);
        acc.send(Signal_type::User_1);
        assert_eq!(acc.peek(), Some(Signal_type::Hangup));
        assert_eq!(acc.pop(), Some(Signal_type::Hangup));
        assert_eq!(acc.peek(), Some(Signal_type::User_1));
        assert_eq!(acc.pop(), Some(Signal_type::User_1));
        assert_eq!(acc.pop(), None);
    }

    #[test]
    fn test_signal_discriminant() {
        assert_eq!(
            Signal_type::Power_failure.get_discriminant(),
            Signal_type::Power_failure as u8
        );
    }
}
