
//#![deny(warnings)]

use futures_util::TryStreamExt;
//use futures_util::stream::try_stream::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {

    //let mut rep_str = format!("res {:?} {:?}",req.method().clone(), &req.uri().path().clone());
    //let whole_body = hyper::body::to_bytes(req.into_body()).await?;
    let mut path : Vec<&str> = req.uri().path().split("/").collect();
    let mut rep_str = format!("{:?}",path);
    let whole_body = hyper::body::to_bytes(req.into_body()).await?;
    rep_str.push_str(" .... ");
    let the_string = String::from_utf8(whole_body.to_vec()).unwrap().to_owned();
    let the_ref_str= &the_string[..];
    rep_str.push_str(the_ref_str);
    Ok(Response::new(Body::from(rep_str)))
}

fn add(x: i32,y: i32) -> i32 {
    x+y
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let fct = add;
    println!("->{}",fct(2,3));

    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(echo)) });
    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}