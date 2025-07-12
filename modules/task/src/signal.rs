use core::fmt::Debug;

/// POSIX signals enumeration
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SignalType {
    /// Hangup detected on controlling terminal or death of controlling process (SIGHUP)
    Hangup = 0,
    /// Interrupt from keyboard (SIGINT)
    Interrupt,
    /// Quit from keyboard (SIGQUIT)
    Quit,
    /// Unused : Illegal Instruction (SIGILL)
    IllegalInstruction,
    /// Trace/breakpoint trap (SIGTRAP)
    Trap,
    /// Abort signal from abort(3) (SIGABRT)
    Abort,
    /// Bus error (bad memory access) (SIGBUS)
    BusError,
    /// Floating-point exception (SIGFPE)
    FloatingPointException,
    /// Kill signal (SIGKILL)
    Kill,
    /// User-defined signal 1 (SIGUSR1)
    User1,
    /// Invalid memory reference (SIGSEGV)
    SegmentationFault,
    /// User-defined signal 2 (SIGUSR2)
    User2,
    /// Broken pipe: write to pipe with no readers (SIGPIPE)
    BrokenPipe,
    /// Timer signal from alarm(2) (SIGALRM)
    Alarm,
    /// Termination signal (SIGTERM)
    Termination,
    /// Stack fault on coprocessor (unused) (SIGSTKFLT)
    StackFault,
    /// Child stopped or terminated (SIGCHLD)
    Child,
    /// Continue if stopped (SIGCONT)
    Continue,
    /// Stop process (SIGSTOP)
    Stop,
    /// Stop typed at terminal (SIGTSTP)
    TerminalStop,
    /// Terminal input for background process (SIGTTIN)
    TerminalInput,
    /// Terminal output for background process (SIGTTOU)
    TerminalOutput,
    /// Urgent condition on socket (4.2BSD) (SIGURG)
    Urgent,
    /// CPU time limit exceeded (4.2BSD) (SIGXCPU)
    CpuTimeLimitExceeded,
    /// File size limit exceeded (4.2BSD) (SIGXFSZ)
    FileSizeLimitExceeded,
    /// Virtual alarm clock (4.2BSD) (SIGVTALRM)
    VirtualAlarm,
    /// Profiling timer expired (SIGPROF)
    ProfilingTimerExpired,
    /// Window resize signal (4.3BSD, Sun) (SIGWINCH)
    WindowResize,
    /// I/O now possible (4.2BSD) (SIGIO)
    IoPossible,
    /// Power failure (System V) (SIGPWR)
    PowerFailure,
    /// Bad system call (SVr4) (SIGSYS)
    BadSystemCall,
}

impl SignalType {
    pub const FIRST: Self = Self::Hangup;
    pub const LAST: Self = Self::BadSystemCall;

    pub const fn get_discriminant(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct SignalAccumulatorType {
    accumulator: u32,
}

impl Default for SignalAccumulatorType {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalAccumulatorType {
    pub const fn new() -> Self {
        Self { accumulator: 0 }
    }

    pub const fn send(&mut self, signal: SignalType) {
        self.accumulator |= 1 << signal as u32;
    }

    pub const fn clear(&mut self, signal: SignalType) {
        self.accumulator &= !(1 << signal as u32);
    }

    pub const fn has_signal(&self, signal: SignalType) -> bool {
        self.accumulator & (1 << signal as u32) != 0
    }

    pub fn peek(&self) -> Option<SignalType> {
        for bit in SignalType::FIRST as u8..=SignalType::LAST as u8 {
            let signal = unsafe { core::mem::transmute::<u8, SignalType>(bit) };

            if self.has_signal(signal) {
                return Some(signal);
            }
        }

        None
    }

    pub fn pop(&mut self) -> Option<SignalType> {
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
        let mut acc = SignalAccumulatorType::new();
        acc.send(SignalType::Interrupt);
        assert!(acc.has_signal(SignalType::Interrupt));
    }

    #[test]
    fn test_clear_signal() {
        let mut acc = SignalAccumulatorType::new();
        acc.send(SignalType::Quit);
        acc.clear(SignalType::Quit);
        assert!(!acc.has_signal(SignalType::Quit));
    }

    #[test]
    fn test_peek_and_pop() {
        let mut acc = SignalAccumulatorType::new();
        acc.send(SignalType::Hangup);
        acc.send(SignalType::User1);
        assert_eq!(acc.peek(), Some(SignalType::Hangup));
        assert_eq!(acc.pop(), Some(SignalType::Hangup));
        assert_eq!(acc.peek(), Some(SignalType::User1));
        assert_eq!(acc.pop(), Some(SignalType::User1));
        assert_eq!(acc.pop(), None);
    }

    #[test]
    fn test_signal_discriminant() {
        assert_eq!(
            SignalType::PowerFailure.get_discriminant(),
            SignalType::PowerFailure as u8
        );
    }
}
