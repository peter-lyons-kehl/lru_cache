use core::hash::{BuildHasher, Hasher};

const SIGNALLED_LENGTH_PREFIX: usize = usize::MAX;

pub fn signal_inject_hash<H: Hasher>(hasher: &mut H, hash: u64) {
    hasher.write_length_prefix(SIGNALLED_LENGTH_PREFIX);
    hasher.write_u64(hash);
}

#[derive(PartialEq, Eq)]
enum SignalState {
    NotSignalled,
    JustSignalled,
    HashJustInjected(u64),
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
    #[inline]
    fn clear_signalled(&mut self) {
        self.state = SignalState::NotSignalled;
    }
}
impl<H: Hasher> Hasher for SignalledInjectionHasher<H> {
    #[inline]
    fn finish(&self) -> u64 {
        if let SignalState::HashJustInjected(hash) = self.state {
            hash
        } else {
            self.hasher.finish()
        }
    }
    /// This does NOT signal, even if it sends the same bytes as `write_length_prefix` and
    /// `write_u64` would when signalling.
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.clear_signalled();
        self.hasher.write(bytes);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.clear_signalled();
        self.hasher.write_u8(i);
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.clear_signalled();
        self.hasher.write_u16(i);
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.clear_signalled();
        self.hasher.write_u32(i);
    }
    fn write_u64(&mut self, i: u64) {
        if self.state == SignalState::JustSignalled {
            self.state = SignalState::HashJustInjected(i);
        } else {
            self.clear_signalled();
        }
        self.hasher.write_u64(i);
    }
    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.clear_signalled();
        self.hasher.write_u128(i);
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.clear_signalled();
        self.hasher.write_usize(i);
    }
    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.clear_signalled();
        self.hasher.write_i8(i);
    }
    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.clear_signalled();
        self.hasher.write_i16(i);
    }
    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.clear_signalled();
        self.hasher.write_i32(i);
    }
    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.clear_signalled();
        self.hasher.write_i64(i);
    }
    #[inline]
    fn write_i128(&mut self, i: i128) {
        self.clear_signalled();
        self.hasher.write_i128(i);
    }
    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.clear_signalled();
        self.hasher.write_isize(i);
    }
    fn write_length_prefix(&mut self, len: usize) {
        if len == SIGNALLED_LENGTH_PREFIX {
            self.state = SignalState::JustSignalled;
        } else {
            self.clear_signalled();
        }
        self.hasher.write_length_prefix(len);
    }
    #[inline]
    fn write_str(&mut self, s: &str) {
        self.clear_signalled();
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
