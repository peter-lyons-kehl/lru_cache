use core::hash::{BuildHasher, Hasher};

const SIGNALLED_LENGTH_PREFIX: usize = usize::MAX;

///
///
/// Do NOT use with standard/third party [Hasher] provided by standard/third party [BuildHasher]
/// implementations for generic purposes. Specifically, with such [Hasher]/[BuildHasher]
/// implementations you
/// - CAN use this when comparing the intended hash to instances of the SAME type, BUT you
/// - CANNOT use this when comparing the intended hash to instances of DIFFERENT type (through
///   [core::borrow::Borrow]) where that other type does NOT use [signal_inject_hash] (and, for
///   example, the intended hash comes from that other type's `Hash::hash` on a [Hasher] created by
///   the same [BuildHasher[]].)
///   
///   Instead, use with [SignalledInjectionHasher] provided by [SignalledInjectionBuildHasher] only.
/// Extra validation of signalling in the user's [core::hash::Hash] implementation is done ONLY in
/// `debug` build (when `debug_assertions` are turned on).
pub fn signal_inject_hash<H: Hasher>(hasher: &mut H, hash: u64) {
    // The order of operations is intentionally different for debug and release. This (hopefully)
    // helps us notice any logical errors or opportunities for improvement in this module earlier.
    #[cfg(debug_assertions)]
    {
        hasher.write_length_prefix(SIGNALLED_LENGTH_PREFIX);
        hasher.write_u64(hash);
    }
    #[cfg(not(debug_assertions))]
    {
        hasher.write_u64(hash);
        hasher.write_length_prefix(SIGNALLED_LENGTH_PREFIX);
    }
}

/// A state machine for a [Hash] implementation to pass a specified hash to [Hasher] - rather than
/// [Hasher] hashing the bytes supplied from [Hash].
///
/// Variants are in order of progression.
#[derive(PartialEq, Eq, Debug)]
enum SignalState {
    NotSignalled,
    #[cfg(debug_assertions)]
    Signalled,
    #[cfg(not(debug_assertions))]
    HashInjected(u64),
    HashSignaled(u64),
}

pub struct SignalledInjectionHasher<H: Hasher> {
    hasher: H,
    state: SignalState,
}
impl<H: Hasher> SignalledInjectionHasher<H> {
    #[inline]
    fn new(hasher: H) -> Self {
        Self {
            hasher,
            state: SignalState::NotSignalled,
        }
    }
    // @TODO if this doesn't optimize away in release, replace with a macro.
    #[inline(always)]
    fn debug_assert_ordinary_write(&self) {
        debug_assert_eq!(self.state, SignalState::NotSignalled);
    }
}
impl<H: Hasher> Hasher for SignalledInjectionHasher<H> {
    #[inline]
    fn finish(&self) -> u64 {
        if let SignalState::HashSignaled(hash) = self.state {
            hash
        } else {
            self.debug_assert_ordinary_write();
            self.hasher.finish()
        }
    }
    /// This does NOT signal, even if it sends the same bytes as `write_length_prefix` and
    /// `write_u64` would when signalling.
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.debug_assert_ordinary_write();
        self.hasher.write(bytes);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.debug_assert_ordinary_write();
        self.hasher.write_u8(i);
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.debug_assert_ordinary_write();
        self.hasher.write_u16(i);
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.debug_assert_ordinary_write();
        self.hasher.write_u32(i);
    }
    fn write_u64(&mut self, i: u64) {
        #[cfg(debug_assertions)]
        if self.state == SignalState::Signalled {
            self.state = SignalState::HashSignaled(i);
        } else {
            self.debug_assert_ordinary_write();
        }
        #[cfg(not(debug_assertions))]
        {
            self.state = SignalState::HashInjected(i);
        }
        self.hasher.write_u64(i);
    }
    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.debug_assert_ordinary_write();
        self.hasher.write_u128(i);
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.debug_assert_ordinary_write();
        self.hasher.write_usize(i);
    }
    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.debug_assert_ordinary_write();
        self.hasher.write_i8(i);
    }
    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.debug_assert_ordinary_write();
        self.hasher.write_i16(i);
    }
    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.debug_assert_ordinary_write();
        self.hasher.write_i32(i);
    }
    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.debug_assert_ordinary_write();
        self.hasher.write_i64(i);
    }
    #[inline]
    fn write_i128(&mut self, i: i128) {
        self.debug_assert_ordinary_write();
        self.hasher.write_i128(i);
    }
    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.debug_assert_ordinary_write();
        self.hasher.write_isize(i);
    }
    fn write_length_prefix(&mut self, len: usize) {
        #[cfg(debug_assertions)]
        if len == SIGNALLED_LENGTH_PREFIX {
            self.state = SignalState::Signalled;
        } else {
            self.debug_assert_ordinary_write();
            self.hasher.write_length_prefix(len);
        }
        #[cfg(not(debug_assertions))]
        if len == SIGNALLED_LENGTH_PREFIX
            && let SignalState::HashInjected(i) = self.state
        {
            self.state = SignalState::HashSignaled(i);
        } else {
            self.hasher.write_length_prefix(len);
        }
    }
    #[inline]
    fn write_str(&mut self, s: &str) {
        self.debug_assert_ordinary_write();
        self.hasher.write_str(s);
    }
}

pub struct SignalledInjectionBuildHasher<H: Hasher, B: BuildHasher<Hasher = H>> {
    build: B,
}
impl<H: Hasher, B: BuildHasher<Hasher = H>> SignalledInjectionBuildHasher<H, B> {
    pub fn new(build: B) -> Self {
        Self { build }
    }
}
impl<H: Hasher, B: BuildHasher<Hasher = H>> BuildHasher for SignalledInjectionBuildHasher<H, B> {
    type Hasher = SignalledInjectionHasher<H>;

    // Required method
    fn build_hasher(&self) -> Self::Hasher {
        SignalledInjectionHasher::new(self.build.build_hasher())
    }
}
