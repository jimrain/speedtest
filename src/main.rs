//! Default Compute@Edge template program.

use fastly::http::{header, Method, StatusCode, HeaderValue};
use fastly::{mime, Error, Request, Response};
use fastly::handle::{ResponseHandle, BodyHandle};
// use fastly::error::BufferKind::{HeaderName, HeaderValue};
use std::collections::HashMap;
use std::collections::hash_map::RandomState;

const CHUNK_SIZE: usize = 4096;
const MAX_FILE_SIZE: usize = 1000000000; // 1Gb
const MIN_FILE_SIZE: usize = 50000; // 50kb
// const FILE_ITERATOR: usize = FILE_SIZE / CHUNK_SIZE;

// const ZERO_VEC: Vec<u8> = vec![0 as u8; CHUNK_SIZE];

/*

const ZERO: &[u8] = &[0, 0];
it'll be like that, but with more zeros
you can `#[rustfmt::skip]` it, and then make each row

 */
/// The entry point for your application.
///
/// This function is triggered when your service receives a client request. It could be used to
/// route based on the request properties (such as method or path), send the request to a backend,
/// make completely new requests, and/or generate synthetic responses.
///
/// If `main` returns an error, a 500 error response will be delivered to the client.

fn main() -> Result<(), Error> {
    let mut req = Request::from_client();

    // Filter request methods...
    match req.get_method() {
        // Allow GET and HEAD requests. Also post for file upload.
        &Method::GET | &Method::HEAD | &Method::POST | &Method::PUT => (),

        // Accept PURGE requests and let it fall through the regular flow
        m if m == "PURGE" => (),

        // Deny anything else.
        _ => {
            let resp = Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD")
                .with_body_text_plain("This method is not allowed\n");
            resp.send_to_client();
            return Ok(());
        }
    };

    // Pattern match on the path.
    match req.get_path() {
        // If request is to the `/` path, send a default response.
        "/" => {
            let resp = Response::from_status(StatusCode::OK)
            .with_content_type(mime::TEXT_HTML_UTF_8)
            .with_body("<iframe src='https://developer.fastly.com/compute-welcome' style='border:0; position: absolute; top: 0; left: 0; width: 100%; height: 100%'></iframe>\n");
            resp.send_to_client();
            return Ok(());
        },

        path if path.starts_with("/__down") => {
            // let qp: Result<HashMap<String, u32>, serde::de::value::Error> = req.get_query();
            let qp: Result<HashMap<String, String>, _> = req.get_query();
            let file_size = match qp {
                Ok(qp) => {
                    let mut fs: usize = qp["bytes"].parse().unwrap();
                    if fs > MAX_FILE_SIZE { fs = MAX_FILE_SIZE; }
                    else if fs < MIN_FILE_SIZE { fs = MIN_FILE_SIZE }
                    fs
                },
                Err(_) => MAX_FILE_SIZE
            };

            let mut rh = ResponseHandle::new();
            rh.insert_header(&header::ACCESS_CONTROL_ALLOW_ORIGIN, &HeaderValue::from_static("*"));
            rh.insert_header(&header::CONTENT_LENGTH, &HeaderValue::from(file_size) );
            rh.set_status(StatusCode::OK);

            let mut streaming_body = rh.stream_to_client(BodyHandle::new());

            let file_iterator = file_size / CHUNK_SIZE;
            println!("File iterator: {}", file_iterator);
            for _ in 0..file_iterator {
                let zero_vec: Vec<u8> = vec![0 as u8; CHUNK_SIZE];
                streaming_body.write_bytes(&zero_vec);
            }
            return Ok(());
        },

        path if path.starts_with("/__up") => {
            for chunk in req.read_body_chunks(CHUNK_SIZE) {
                let mut chunk = chunk.unwrap();
                println!("Chunk len: {}", chunk.len())
            }
            Ok(())
        },

        // Catch all other requests and return a 404.
        _ => {
            let resp = Response::from_status(StatusCode::NOT_FOUND)
                .with_body_text_plain("The page you requested could not be found\n");
            resp.send_to_client();
            return Ok(());
        },
    }
}
