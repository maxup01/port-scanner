use port_scanner::App;

#[tokio::main]
async fn main() {
    let app = App::init();
    app.run().await;
}
