use clap::Parser;
use num_primes::{BigUint, Generator, Verification};
use num_traits::ToPrimitive;
use rand::Rng;

#[derive(Parser, Debug)]
struct Options {
    #[arg(short, long, default_value = None, help = "The prime order q")]
    q_prime: Option<u64>,
}

fn verify_generator(element: &BigUint, order: &BigUint) -> Result<(), Box<dyn std::error::Error>> {
    let mut powers = vec![element.clone()];
    let two = BigUint::from(2_u64);
    let limit = order.to_u128().ok_or("Order is not a u128")?;
    for _ in 1..limit {
        powers.push(powers.last().ok_or("No last element")?.modpow(&two, order));
    }
    if powers.len() as u128 != limit {
        return Err(format!("Element is not a generator or order {}", order)
            .to_string()
            .into());
    }
    Ok(())
}

fn verify_prime(order: &BigUint) -> Result<(), Box<dyn std::error::Error>> {
    if !Verification::is_prime(order) {
        return Err(format!("Order {} is not prime", order).to_string().into());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the prime order q
    let options = Options::parse();
    let q = options
        .q_prime
        .map(BigUint::from)
        .unwrap_or(Generator::safe_prime(32));

    println!("Prime order q: {}", q);

    verify_prime(&q)?;

    let mut rng = rand::thread_rng();
    let limit = q.to_u128().ok_or("Order is not a u128")?;

    // Generate a random element g in the group
    let g: BigUint = rng.gen_range(2..=limit - 1).into();

    // Generate a random element h in the group
    let h: BigUint = rng.gen_range(2..=limit - 1).into();

    println!("Element g: {}", g);

    // Ensure g and h are generators of the group
    verify_generator(&g, &q)?;
    verify_generator(&h, &q)?;

    let json = serde_json::json!({
        "g": g.to_string(),
        "h": h.to_string(),
        "q": q.to_string(),
    });
    println!("{}", json);

    Ok(())
}
