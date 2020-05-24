use std::{io, net::SocketAddr};
use futures::{future::join_all, prelude::*};
use tarpc::{client, context};
use tokio_serde::formats::Json;


#[tokio::main]
async fn main() -> io::Result<()> {
    let mut res = vec![];
    for port in 8000..8008 {
        let server_addr = format!("127.0.0.1:{}", port);
        let server_addr = server_addr.parse::<SocketAddr>().unwrap();

        let th = tokio::spawn(async move {
            let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default()).await.unwrap();
            let mut client = service::WorldClient::new(client::Config::default(), transport).spawn().unwrap();
            let hello = client.hello(context::current(), "wiki".into());
            hello.await
        });
        res.push(th);
    }

    let out = join_all(res).await;

    for o in out.into_iter() {
        println!("{:?}", o);
    }

    Ok(())
}
