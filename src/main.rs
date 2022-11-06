#[async_std::main]
async fn main() {
    println!("Hello, world!");

    let body = surf::get("https://archive.org/details/xboxcds?&sort=titleSorter&scroll=1&page=1")
        .recv_string()
        .await
        .expect("");

    println!("Body {}", body);
}
