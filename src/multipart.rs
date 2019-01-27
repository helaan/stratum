use actix_web::http::header::ContentDisposition;
use actix_web::multipart::{Multipart, MultipartItem};
use actix_web::{dev::Payload, error, Error, FutureResponse};
use bytes::BytesMut;
use futures::future::{result, Future, FutureResult};
use futures::stream::Stream;
use std::collections::HashMap;
use std::str;
use std::str::FromStr;

pub struct MultipartForm {
    pub content: HashMap<String, (String, ContentDisposition, BytesMut)>,
}

impl MultipartForm {
    pub fn get(&self, key: &str) -> Result<&(String, ContentDisposition, BytesMut), Error> {
        self.content
            .get(key)
            .ok_or_else(|| error::ErrorBadRequest(format!("Could not find {}", key)))
    }

    pub fn get_content_str(&self, key: &str) -> Result<&str, Error> {
        str::from_utf8(self.get(key)?.2.as_ref()).map_err(|e| {
            error::ErrorBadRequest(format!("Could not parse {} as string: {}", key, e))
        })
    }

    pub fn get_parsed_content<T: FromStr>(&self, key: &str) -> Result<T, Error> {
        self.get_content_str(key)?
            .parse::<T>()
            .map_err(|_| error::ErrorBadRequest(format!("Could not parse {}", key)))
    }
}

fn parse_item(
    item: MultipartItem<Payload>,
) -> Box<Stream<Item = (String, Option<ContentDisposition>, BytesMut), Error = Error>> {
    match item {
        MultipartItem::Field(field) => {
            let mime = field.content_type().to_string();
            let cont_disp = field.content_disposition();
            let f = field
                .fold(
                    BytesMut::new(),
                    |mut acc, bytes| -> FutureResult<_, error::MultipartError> {
                        acc.extend_from_slice(&bytes);
                        result(Ok(acc))
                    },
                )
                .map(|bytes| (mime, cont_disp, bytes))
                .map_err(error::ErrorInternalServerError);
            Box::new(f.into_stream())
        }
        MultipartItem::Nested(nest) => Box::new(
            nest.map(parse_item)
                .map_err(error::ErrorInternalServerError)
                .flatten(),
        ),
    }
}

pub fn parse_multipart(mp: Multipart<Payload>) -> FutureResponse<MultipartForm> {
    Box::new(
        mp.map_err(error::ErrorInternalServerError)
            .map(parse_item)
            .flatten()
            .fold(HashMap::new(), |mut acc, item| -> FutureResult<_, Error> {
                if let Some(cd) = item.1 {
                    log::debug!("Multipart Form field: {}", cd.to_string());
                    if let Some(name) = cd.get_name() {
                        acc.insert(name.into(), (item.0, cd, item.2));
                        result(Ok(acc))
                    } else {
                        result(Err(error::ErrorBadRequest(
                            "Missing form name in ContentDisposition",
                        )))
                    }
                } else {
                    /* Browsers always add a ContentDisposition header, but Actix removes it when
                     * no file was uploaded. We'll just ignore these entries.
                     * TODO henk: check in upstream why this happens
                     */
                    //result(Err(error::ErrorBadRequest("Missing ContentDisposition")))
                    result(Ok(acc))
                }
            })
            .map(|h| MultipartForm { content: h }),
    )
}
