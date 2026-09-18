use port_scanner::App;

#[tokio::main]
async fn main() {
    let _ = App::run().await;
}
