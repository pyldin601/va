mod audio;
mod config;

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use actix_web::{web, App, Error, HttpResponse, HttpServer};
use bytes::Bytes;
use futures_util::Stream;
use tokio::sync::broadcast;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use vosk::Model;

#[derive(Clone)]
struct AppState {
    sender: broadcast::Sender<String>,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind_addr = config::get_bind_addr();
    let model_path = config::get_vosk_model_path();

    let model = Arc::new(Model::new(model_path).ok_or("Failed to load VOSK model")?);

    let (sender, _) = broadcast::channel(128);
    audio::spawn_shared_recognizer_stream(Arc::clone(&model), sender.clone())?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                sender: sender.clone(),
            }))
            .route("/text", web::get().to(text_stream))
    })
    .bind(bind_addr)?
    .run()
    .await?;

    Ok(())
}

async fn text_stream(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let receiver = state.sender.subscribe();
    let stream = SseStream::new(receiver);

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream))
}

struct SseStream {
    inner: BroadcastStream<String>,
}

impl SseStream {
    fn new(receiver: broadcast::Receiver<String>) -> Self {
        Self {
            inner: BroadcastStream::new(receiver),
        }
    }
}

impl Stream for SseStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            return match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(text))) => {
                    let body = format!("data: {}\n\n", sse_escape(&text));
                    Poll::Ready(Some(Ok(Bytes::from(body))))
                }
                Poll::Ready(Some(Err(BroadcastStreamRecvError::Lagged(_)))) => continue,
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            };
        }
    }
}

fn sse_escape(text: &str) -> String {
    text.replace('\n', "\ndata: ")
}
