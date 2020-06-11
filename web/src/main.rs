use tide::Redirect;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    // FIXME: a bit dangerous to use such relative paths
    app.at("/pkg/").serve_dir("../wasm/pkg/")?;
    app.at("/web/").serve_dir("../wasm/web/")?;
    app.at("/").get(Redirect::new("/web/index.html"));
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}