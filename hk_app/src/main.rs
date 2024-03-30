use hk_core::nat_utils;

#[tokio::main]
async fn main() {
    match nat_utils::get_nat_type().await.unwrap() {
        nat_utils::NatType::Common => println!("Common NAT"),
        nat_utils::NatType::Symmetric => println!("Symmetric NAT"),
    };
}
