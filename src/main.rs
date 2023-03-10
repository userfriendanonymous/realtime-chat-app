mod server;
 
#[actix_web::main]
async fn main() -> Result<(), ()> {
    match server::serve().await {
        Ok(_) => {},
        Err(err) => println!("{err:?}"),
    };

    Ok(())
}