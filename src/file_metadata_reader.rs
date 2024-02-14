use gloo_net::http::Method;
use rexif::parse_buffer;
use web_sys::File;
use gloo_net::http::{RequestBuilder};
use crate::ImageMetadata;
use gloo_net::Error;
use std::io::Cursor;
use image::io::Reader;

pub async fn binary_from_file(file: File) -> Result<Vec<u8>, Error> {
    let request = RequestBuilder::new("url").method(Method::PUT).body(&file).expect("request expected");
    request.binary().await
}

pub fn update_image_metadata_from_binary(source: ImageMetadata, file_binary: Vec<u8>) -> Option<ImageMetadata> {
    let mut im = source.clone();
    im.size = file_binary.len() as f64;
    let reader = Reader::new(Cursor::new(file_binary.clone()))
        .with_guessed_format()
        .expect("Cursor io never fails");
    let dimensions = reader.into_dimensions();
    match dimensions {
        Ok(di) => {
            im.set_dimensions(di);
        },
        Err(_) => (),
    }

    let mut all_tags: Vec<(String, String)>  = vec![].into();

    let _ = match parse_buffer(&file_binary.clone()) {
        Ok(result) => {
             for entry in result.entries {
                if entry.tag.to_string() == "GPS latitude" {
                    im.set_lat(entry.value_more_readable.to_string());
                }
                if entry.tag.to_string() == "GPS longitude" {
                    im.set_long(entry.value_more_readable.to_string());
                }
                all_tags.push((entry.tag.to_string(), entry.value_more_readable.to_string()));
            };
        },
        Err(_) => {
            im.set_lat("No tags found".to_string());
            im.set_long("No tags found".to_string());
        },
    };
    im.all_tags = all_tags.clone();
    Some(im)
}

pub async fn parse_image_metadata(file: File) -> Option<ImageMetadata> {
    match binary_from_file(file.clone()).await {
        Ok(body) => {
            let mut im = ImageMetadata::new(file.clone());
            im.set_raw_binary(binary_from_file(file.clone()).await.expect("binary expected"));

            let reader = Reader::new(Cursor::new(binary_from_file(file.clone()).await.expect("binary expected")))
                .with_guessed_format()
                .expect("Cursor io never fails");
            let dimensions = reader.into_dimensions();
            match dimensions {
                Ok(di) => {
                    im.set_dimensions(di);
                },
                Err(_) => (),
            }

            return match parse_buffer(&body) {
                Ok(result) => {
                    let mut all_tags: Vec<(String, String)>  = vec![].into(); 
                    for entry in result.entries {
                        all_tags.push((entry.tag.to_string(), entry.value_more_readable.to_string()));
                        if entry.tag.to_string() == "GPS latitude" {
                            im.set_lat(entry.value_more_readable.to_string());
                        }
                        if entry.tag.to_string() == "GPS longitude" {
                            im.set_long(entry.value_more_readable.to_string());
                        }
                    };
                    im.all_tags = all_tags;
                    Some(im)
                },
                Err(_) => Some(im)
            }
        },
        Err(_) => None
    }
}