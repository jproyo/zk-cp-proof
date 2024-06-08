use zk_client::grpc::zkp_auth::RegisterRequest;

fn main() {
    println!("Hello, world!");

    RegisterRequest {
        username: "test".to_string(),
        password: "test".to_string(),
    };
}
