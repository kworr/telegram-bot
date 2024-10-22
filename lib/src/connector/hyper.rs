use std::io::{Cursor, Read};
use std::path::Path;
use std::pin::Pin;
use std::str::FromStr;

use bytes::Bytes;
use futures::{Future, FutureExt};
use http_body_util::{BodyExt, Full};
use hyper::{
    header::{HeaderValue, CONTENT_TYPE},
    http::Error as HttpError,
    Method, Request, Uri,
};
#[cfg(feature = "rustls")]
use hyper_rustls::HttpsConnectorBuilder;
#[cfg(feature = "openssl")]
use hyper_util::{
    client::legacy::{connect::Connect, Client},
    rt::TokioExecutor,
};
use multipart_2021::client::lazy::Multipart;
use telegram_bot_raw::{
    Body as TelegramBody, HttpRequest, HttpResponse, Method as TelegramMethod, MultipartValue, Text,
};

use super::Connector;
use crate::errors::Error;

static JSON_MIME_TYPE: HeaderValue = HeaderValue::from_static("application/json");

#[derive(Debug)]
pub struct HyperConnector<C>(Client<C, Full<Bytes>>);

enum MultipartTemporaryValue {
    Text(Text),
    Data { file_name: Text, data: Bytes },
}

impl<C> HyperConnector<C> {
    pub fn new(client: Client<C, Full<Bytes>>) -> Self {
        HyperConnector(client)
    }
}

impl<C: Connect + std::fmt::Debug + 'static + Clone + Send + Sync> Connector for HyperConnector<C> {
    fn request(
        &self,
        token: &str,
        req: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>> {
        let uri = Uri::from_str(&req.url.url(token));
        let client = self.0.clone();

        let future = async move {
            let uri = uri.map_err(HttpError::from).map_err(Error::from)?;

            let method = match req.method {
                TelegramMethod::Get => Method::GET,
                TelegramMethod::Post => Method::POST,
            };

            let mut http_request = Request::builder().method(method).uri(uri);

            let request = match req.body {
                TelegramBody::Empty => http_request.body(Full::new(Bytes::new())),
                TelegramBody::Json(body) => {
                    http_request
                        .headers_mut()
                        .map(move |headers| headers.insert(CONTENT_TYPE, JSON_MIME_TYPE.clone()));
                    http_request.body(Full::new(body.into()))
                }
                TelegramBody::Multipart(parts) => {
                    let mut fields = Vec::new();
                    for (key, value) in parts {
                        match value {
                            MultipartValue::Text(text) => {
                                fields.push((key, MultipartTemporaryValue::Text(text)))
                            }
                            MultipartValue::Path { file_name, path } => {
                                let file_name = file_name
                                    .or_else(|| {
                                        AsRef::<Path>::as_ref(&path)
                                            .file_name()
                                            .and_then(|s| s.to_str())
                                            .map(Into::into)
                                    })
                                    .ok_or(Error::InvalidMultipartFilename)?;

                                let data = tokio::fs::read(path).await.map_err(Error::from)?;
                                fields.push((
                                    key,
                                    MultipartTemporaryValue::Data {
                                        file_name,
                                        data: data.into(),
                                    },
                                ))
                            }
                            MultipartValue::Data { file_name, data } => fields
                                .push((key, MultipartTemporaryValue::Data { file_name, data })),
                        }
                    }

                    let mut prepared = {
                        let mut part = Multipart::new();
                        for (key, value) in &fields {
                            match value {
                                MultipartTemporaryValue::Text(text) => {
                                    part.add_text(*key, text.as_str());
                                }
                                MultipartTemporaryValue::Data { file_name, data } => {
                                    part.add_stream(
                                        *key,
                                        Cursor::new(data),
                                        Some(file_name.as_str()),
                                        None,
                                    );
                                }
                            }
                        }
                        part.prepare().map_err(|err| err.error)
                    }
                    .map_err(Error::from)?;

                    let boundary = prepared.boundary();

                    let content_type = format!("multipart/form-data;boundary={boundary}")
                        .parse()
                        .map_err(HttpError::from)
                        .map_err(Error::from)?;
                    if let Some(headers) = http_request.headers_mut() {
                        headers.insert(CONTENT_TYPE, content_type);
                    }

                    let mut bytes = Vec::new();
                    prepared.read_to_end(&mut bytes).map_err(Error::from)?;
                    http_request.body(bytes.into())
                }
                body => panic!("Unknown body type {body:?}"),
            }
            .map_err(Error::from)?;

            let response = client.request(request).await.map_err(Error::from)?;
            let body = response
                .into_body()
                .collect()
                .await
                .map(|body| body.to_bytes().to_vec())
                .unwrap_or_default();

            Ok::<HttpResponse, Error>(HttpResponse { body: Some(body) })
        };

        future.boxed()
    }
}

pub fn default_connector() -> Result<Box<dyn Connector>, Error> {
    #[cfg(any(feature = "rustls-native-roots", feature = "rustls-webpki-roots"))]
    let connector = {
        let builder = hyper_rustls::HttpsConnectorBuilder::new();

        #[cfg(feature = "rustls-native-roots")]
        let builder = builder
            .with_native_roots()
            .expect("no native root CA certificates found");

        #[cfg(feature = "rustls-webpki-roots")]
        let builder = builder.with_webpki_roots();

        builder.https_only().enable_http1().enable_http2().build()
    };

    #[cfg(feature = "openssl")]
    let connector = {
        let mut connector = hyper_tls::HttpsConnector::new();
        connector.https_only(true);
        connector
    };

    Ok(Box::new(HyperConnector::new(
        Client::builder(TokioExecutor::new()).build(connector),
    )))
}
