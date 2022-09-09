use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

struct UpperAlphanumeric;

impl Distribution<char> for UpperAlphanumeric {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

        let range = Uniform::new(0, CHARSET.len());
        let n = range.sample(rng);
        unsafe { char::from_u32_unchecked(CHARSET[n] as u32) }
    }
}

pub fn generate_join_code<R: Rng + ?Sized>(rng: &mut R) -> String {
    (0..7)
        .map(|_| rng.sample(UpperAlphanumeric) as char)
        .collect()
}
