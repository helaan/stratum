use actix_web::http::header::ContentDisposition;
use actix_web::multipart::{Multipart, MultipartItem};
use actix_web::{dev::Payload, error, Error, FutureResponse};
use bytes::BytesMut;
use futures::future::{result, Future, FutureResult};
use futures::stream::Stream;
use std::collections::HashMap;

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

pub fn parse_multipart(
    mp: Multipart<Payload>,
) -> FutureResponse<HashMap<String, (String, ContentDisposition, BytesMut)>> {
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
                    result(Err(error::ErrorBadRequest("Missing ContentDisposition")))
                }
            }),
    )
}
