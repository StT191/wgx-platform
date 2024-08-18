
pub use getrandom;
pub use fastrand::{self, Rng};


pub fn entropy() -> u64 {

    // try with getrandom
    use std::mem::{MaybeUninit, transmute};

    unsafe {
        // SAFETY: array of uninits is valid
        let mut bytes_uninit: [MaybeUninit<u8>; 8] = MaybeUninit::uninit().assume_init();

        if getrandom::getrandom_uninit(&mut bytes_uninit).is_ok() {
            // SAFETY: bytes can be assumed init after getrandom succeeds
            let bytes: [u8; 8] = transmute(bytes_uninit);
            return u64::from_ne_bytes(bytes)
        }

    }

    // fallback

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use crate::time::Instant;

    let mut hasher = DefaultHasher::new();
    Instant::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    let hash = hasher.finish();

    (hash << 1) | 1
}


// convenience method to instatiate a Rng with entropy

pub trait WithEntropy {
    fn with_entropy() -> Self;
}

impl WithEntropy for Rng {
    fn with_entropy() -> Self {
        Self::with_seed(entropy())
    }
}


#[cfg(test)]
mod tests {

    use super::{Rng, WithEntropy};

    #[test]
    fn ranges() {

        let mut rng = Rng::with_entropy();

        let num = rng.usize(1..2);
        assert_eq!(num, 1);

        let num = rng.isize(-3..-2);
        assert_eq!(num, -3);
    }
}