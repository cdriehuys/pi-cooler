#[macro_use]
extern crate lazy_static;

use std::{
    convert::Infallible,
    error::Error,
    fs,
    net::SocketAddr,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use prometheus_client::{
    encoding::text::encode,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};

const CPU_TEMP_FILE: &str = "/sys/class/thermal/thermal_zone0/temp";

lazy_static! {
    static ref CPU_TEMP: Family<String, Gauge<f64, AtomicU64>> = Family::default();
}

async fn encode_metrics(registry: Arc<Registry>) -> Result<Response<Body>, Infallible> {
    let mut encoded_metrics = vec![];
    encode(&mut encoded_metrics, &registry).unwrap();

    Ok(Response::new(encoded_metrics.into()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut registry = <Registry>::default();
    registry.register(
        "cpu_temp",
        "Current CPU temperature in Celsius",
        Box::new(CPU_TEMP.clone()),
    );

    let cpu_temp_task = tokio::task::spawn(async { poll_cpu_temp(CPU_TEMP_FILE).await });

    let mut encoded_metrics = vec![];
    encode(&mut encoded_metrics, &registry).unwrap();

    println!("{}", String::from_utf8(encoded_metrics)?);

    let addr: SocketAddr = "0.0.0.0:8000".parse()?;

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let registry_ref = Arc::new(registry);
    let make_svc = make_service_fn(move |_conn| {
        let registry_ref = registry_ref.clone();
        async move {
            // service_fn converts our function into a `Service`
            Ok::<_, Infallible>(service_fn(move |_req: Request<Body>| {
                let registry_ref = registry_ref.clone();
                encode_metrics(registry_ref)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    cpu_temp_task.await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    registry: Arc<Registry>,
}

fn read_cpu_temp(path: &str) -> Result<f64, Box<dyn Error>> {
    let raw_temp = fs::read_to_string(path)?;
    let parsed_temp_millis: f64 = raw_temp.trim().parse()?;

    Ok(parsed_temp_millis / 1000_f64)
}

async fn poll_cpu_temp(path: &str) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        match read_cpu_temp(path) {
            Ok(temp) => {
                println!("CPU Temp: {:.1} C", temp);
                CPU_TEMP.get_or_create(&"zone0".to_owned()).set(temp);
            }
            Err(err) => {
                eprintln!("Failed to read CPU temp: {:?}", err);
            }
        };
    }
}
