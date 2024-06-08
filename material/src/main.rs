use num_bigint::{BigInt, ToBigInt};
use num_traits::{One, Zero};
use rand::Rng;

fn main() {
    // Define the prime order q
    let q: u64 = 23; // Example prime number
    let mut rng = rand::thread_rng();

    // Generate a random element g in the group
    let g: BigInt = rng.gen_range(2..=q - 1).into();
    println!("g: {}", g);

    // Generate a random element h in the group
    let h: BigInt = rng.gen_range(2..=q - 1).into();
    println!("h: {}", h);

    // Ensure g and h are generators of the group
    assert!(is_generator(&g, q));
    assert!(is_generator(&h, q));

    // Function to check if an element is a generator of the group
    fn is_generator(element: &BigInt, order: u64) -> bool {
        let mut powers = vec![element.clone()];
        for _ in 1..order {
            powers.push(
                powers
                    .last()
                    .unwrap()
                    .modpow(&2.to_bigint().unwrap(), &order.to_bigint().unwrap()),
            );
        }
        powers.len() as u64 == order
    }
}
