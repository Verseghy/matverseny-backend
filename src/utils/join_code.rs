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
        CHARSET[n] as char
    }
}

pub fn generate_join_code<R: Rng + ?Sized>(rng: &mut R) -> String {
    (0..6).map(|_| rng.sample(UpperAlphanumeric)).collect()
}

#[test]
fn join_code_length() {
    let code = generate_join_code(&mut rand::thread_rng());
    assert_eq!(code.len(), 6);
}
