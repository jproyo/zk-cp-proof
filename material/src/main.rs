use clap::Parser;
use num_primes::{BigUint, Generator, Verification};
use num_traits::{One, ToPrimitive, Zero};
use rand::Rng;
use tokio::time::Instant;

#[derive(Parser, Debug)]
struct Options {
    #[arg(short, long, default_value = None, help = "The prime order q")]
    q_prime: Option<u64>,
}

fn verify_generator(element: &BigUint, order: &BigUint) -> Result<(), Box<dyn std::error::Error>> {
    let two = BigUint::from(2_u64);
    let limit = order.to_u128().ok_or("Order is not a u128")?;
    let mut last = element.clone();
    let mut count = 1;
    for _ in 1..limit {
        last = last.modpow(&two, order);
        if last.is_one() || last.is_zero() {
            return Err(format!("Element {} is not a generator", element)
                .to_string()
                .into());
        }
        count += 1;
    }
    if count != limit {
        return Err(format!("Element {} is not a generator", element)
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
        .unwrap_or(Generator::safe_prime(16));

    println!("Prime order q: {}", q);

    verify_prime(&q)?;
    let limit = q.to_u128().ok_or("Order is not a u128")?;

    let current_time = Instant::now();

    let (g, h) = loop {
        let r = generate_group(&q, limit);
        if r.is_ok() {
            break r.unwrap();
        }
        if current_time.elapsed().as_secs() > 10 {
            return Err("Too many attempts".into());
        }
    };

    let json = serde_json::json!({
        "g": g.to_string(),
        "h": h.to_string(),
        "q": q.to_string(),
    });
    println!("{}", json);

    Ok(())
}

fn generate_group(
    q: &BigUint,
    limit: u128,
) -> Result<(BigUint, BigUint), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();

    // Generate a random element g in the group
    let g: BigUint = rng.gen_range(2..=limit - 1).into();

    // Generate a random element h in the group
    let h: BigUint = rng.gen_range(2..=limit - 1).into();

    println!("Element g: {}", g);

    // Ensure g and h are generators of the group
    verify_generator(&g, q)?;
    verify_generator(&h, q)?;
    Ok((g, h))
}
