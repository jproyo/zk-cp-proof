use num_primes::{BigUint, Generator, Verification};
use num_traits::{One, ToPrimitive, Zero};
use rand::Rng;
use tokio::sync::oneshot;
use tokio::time::Duration;

use crate::domain::material::{Material, MaterialGenerator, PrimeOrder};

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

fn generate_group(
    q: &BigUint,
    limit: u128,
) -> Result<(BigUint, BigUint), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();

    // Generate a random element g in the group
    let g: BigUint = rng.gen_range(2..=limit - 1).into();

    // Generate a random element h in the group
    let h: BigUint = rng.gen_range(2..=limit - 1).into();

    // Ensure g and h are generators of the group
    verify_generator(&g, q)?;
    verify_generator(&h, q)?;
    Ok((g, h))
}

pub(crate) struct DefaultMaterialGenerator;

#[async_trait::async_trait]
impl MaterialGenerator for DefaultMaterialGenerator {
    async fn generate(&self, q: Option<PrimeOrder>) -> anyhow::Result<Material> {
        let q = q.map(Into::into).unwrap_or(Generator::safe_prime(16));

        verify_prime(&q).map_err(|e| anyhow::anyhow!("{e}"))?;
        let limit = q
            .to_u128()
            .ok_or("Order is not a u128")
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let (timeout_tx, timeout_rx) = oneshot::channel();
        let (group_tx, group_rx) = oneshot::channel();
        let timeout_task = async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            timeout_tx.send(()).unwrap();
        };

        let task = async move {
            loop {
                let r = generate_group(&q, limit);
                if let Ok(r) = r {
                    group_tx
                        .send(r)
                        .map_err(|_| anyhow::anyhow!("Could not send value to channel"))?;
                    break;
                }
            }
            Ok::<(), anyhow::Error>(())
        };
        let t = tokio::spawn(timeout_task);
        tokio::spawn(task);
        let r = tokio::select! {
            _ = timeout_rx => {
                return Err(anyhow::anyhow!("Timeout in generating group"))
            }
            result = group_rx => {
                t.abort();
                result
            }
        }?;

        Ok(Material::builder().g(r.0).h(r.1).build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_material_generator() {
        let generator = DefaultMaterialGenerator;
        let material = generator.generate(None).await;
        assert!(material.is_ok());
    }

    #[test]
    fn test_verify_generator() {
        let q = BigUint::from(23_u64);
        let g = BigUint::from(5_u64);
        let result = verify_generator(&g, &q);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_prime() {
        let q = BigUint::from(23_u64);
        let result = verify_prime(&q);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_group() {
        let q = BigUint::from(23_u64);
        let limit = q.to_u128().unwrap();
        let result = generate_group(&q, limit);
        assert!(result.is_ok());
    }
}
