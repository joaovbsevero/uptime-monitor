use poem::{Endpoint, IntoResponse, Request, Response};
use tracing::{info, span, Level};

pub(crate) async fn log<E: Endpoint>(next: E, req: Request) -> poem::Result<Response> {
    let span = span!(Level::INFO, "request-span");
    let _guard = span.enter();

    let request = format!("Request: {}", req.uri().path());
    info!(request);

    let res = next.call(req).await;
    let (response, res) = match res {
        Ok(resp) => {
            let resp = resp.into_response();
            let res_info = format!("Response: {}", resp.status());
            (res_info, Ok(resp))
        }
        Err(err) => {
            let res_info = format!("An error has ocurrred: {err}");
            (res_info, Err(err))
        }
    };
    info!(response);
    res
}