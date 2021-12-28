use axum::{extract::Extension, routing::post, AddExtensionLayer, Json, Router};
use rs_ws281x::{ChannelBuilder, ControllerBuilder, StripType};
use serde::Deserialize;
use std::{net::Ipv6Addr, sync::Arc, thread};
use tokio::sync::{mpsc, oneshot};

const CHANNEL_INDEX: usize = 1;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<usize> = std::env::args()
        .skip(1)
        .take(2)
        .map(|p| p.parse())
        .collect::<Result<_, _>>()?;
    let mut args = args.into_iter();
    let chan_index = args.next().unwrap_or(CHANNEL_INDEX);
    let pin = args.next().unwrap_or(19);

    let (boot_tx, boot_rx) = oneshot::channel();
    let (data_tx, mut data_rx) = mpsc::channel::<PixelBatch>(100);
    thread::spawn(move || {
        let channel = ChannelBuilder::new()
            .pin(pin as i32)
            .count(10)
            .strip_type(StripType::Ws2811Rgb)
            .brightness(55)
            .count(300)
            .build();
        let maybe_controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(chan_index, channel)
            .build();
        let mut controller = match maybe_controller {
            Ok(c) => c,
            Err(e) => {
                boot_tx.send(Err(e)).unwrap();
                return;
            }
        };
        boot_tx.send(Ok(())).unwrap();

        while let Some(data) = data_rx.blocking_recv() {
            println!("instruction: {:?}", data);
            let pixels = controller.leds_mut(chan_index);
            let mut i = data.offset.unwrap_or(0);
            for (p, (r, g, b)) in data.pixels.iter().cycle().enumerate() {
                if p > data.pixels.len() && !data.r#loop {
                    break;
                }
                if i >= pixels.len() {
                    if !data.r#loop {
                        println!("could not write all values. end of strand reached.");
                    }
                    break;
                }
                pixels[i] = [*r, *g, *b, 0];
                i += data.step.unwrap_or(1);
            }
            controller
                .render()
                .and_then(|_| controller.wait())
                .map_err(|e| println!("Failed to render: {}", e))
                .ok();
        }
    });
    boot_rx.await??;

    let app = Router::new()
        .route("/batch", post(batch))
        .layer(AddExtensionLayer::new(Arc::new(data_tx)));

    let addr = (Ipv6Addr::UNSPECIFIED, 3000).into();
    println!("Running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct PixelBatch {
    pixels: Vec<(u8, u8, u8)>,
    offset: Option<usize>,
    step: Option<usize>,
    #[serde(default)]
    r#loop: bool,
}

async fn batch(
    Json(payload): Json<PixelBatch>,
    Extension(tx): Extension<Arc<mpsc::Sender<PixelBatch>>>,
) {
    tx.try_send(payload).ok();
}
