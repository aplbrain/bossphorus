#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod data_manager;
mod intern;

use data_manager::data_manager::{ChunkedBloscFileDataManager, DataManager, Vector3};
use ndarray::Array;
use rocket::data::Data;
use rocket::http::RawStr;
use rocket::response::{status, Stream};
use rocket::Request;
use rocket_contrib::json::Json;
use serde_derive::{Deserialize, Serialize};
use std::io::{Cursor, Read};

#[derive(Serialize, Deserialize, Debug)]
struct ChannelMetadata {
    /// Metadata corresponding to a channel.
    ///
    /// A struct holder for the metadata returned by Bosslikes at the
    /// channel-metadata endpoint.
    name: String,
    description: String,
    experiment: String,
    collection: String,
    default_time_sample: u64,
    _type: String,
    base_resolution: u64,
    datatype: String,
    creator: String,
    sources: Vec<String>,
    downsample_status: String,
    related: Vec<String>,
}

/// Convert a colon-delimited extents variable into a `Vec<u64>` of len=2.
///
/// # Arguments:
///
/// * `string_value` - A string that contains two integers separated by a colon
fn colon_delim_str_to_extents(string_value: &RawStr) -> Vec<u64> {
    string_value
        .split(":")
        .map(|t| t.parse::<u64>().unwrap())
        .collect()
}

#[get("/collection/<collection>/experiment/<experiment>/channel/<channel>")]
fn get_channel_metadata(
    collection: &RawStr,
    experiment: &RawStr,
    channel: &RawStr,
) -> Json<ChannelMetadata> {
    Json(ChannelMetadata {
        name: channel.to_string(),
        description: "".to_string(),
        experiment: experiment.to_string(),
        collection: collection.to_string(),
        default_time_sample: 0,
        _type: "image".to_string(),
        base_resolution: 0,
        datatype: "uint8".to_string(),
        creator: "bossphorus_cache".to_string(),
        sources: vec![],
        downsample_status: "DOWNSAMPLED".to_string(),
        related: vec![],
    })
}

#[get("/cutout/<collection>/<experiment>/<channel>/<res>/<xs>/<ys>/<zs>")]
fn download(
    collection: &RawStr,
    experiment: &RawStr,
    channel: &RawStr,
    res: u8,
    xs: &RawStr,
    ys: &RawStr,
    zs: &RawStr,
) -> Result<Stream<Cursor<Vec<u8>>>, std::io::Error> {
    // Parse out the extents:
    let x_extents: Vec<u64> = colon_delim_str_to_extents(xs);
    let y_extents: Vec<u64> = colon_delim_str_to_extents(ys);
    let z_extents: Vec<u64> = colon_delim_str_to_extents(zs);

    // Try to convert to origin-and-shape:
    let origin = Vector3 {
        x: x_extents[0],
        y: y_extents[0],
        z: z_extents[0],
    };
    let destination = Vector3 {
        x: x_extents[1],
        y: y_extents[1],
        z: z_extents[1],
    };

    // TODO: Assert that shape is positive

    // Perform the data-read:
    let fm = ChunkedBloscFileDataManager::new(
        format!(
            "upload/{collection}/{experiment}/{channel}/{res}",
            collection = collection,
            experiment = experiment,
            channel = channel,
            res = res
        ),
        Vector3 {
            x: 512,
            y: 512,
            z: 64,
        },
    );

    let result = fm
        .get_data(
            format!("bossdb://{}/{}/{}", collection, experiment, channel),
            res,
            origin,
            destination,
        )
        .into_raw_vec();

    let ctx = blosc::Context::new();
    let compressed: blosc::Buffer<u8> = ctx.compress(&result[..]);

    let cur: Cursor<Vec<u8>> = Cursor::new(compressed.into());
    let response = Stream::from(cur);

    Ok(response)
}

#[post(
    "/cutout/<collection>/<experiment>/<channel>/<res>/<xs>/<ys>/<zs>",
    data = "<data>"
)]
fn upload(
    data: Data,
    collection: &RawStr,
    experiment: &RawStr,
    channel: &RawStr,
    res: u8,
    xs: &RawStr,
    ys: &RawStr,
    zs: &RawStr,
) -> status::Created<String> {
    // Parse out the extents:
    let x_extents: Vec<u64> = colon_delim_str_to_extents(xs);
    let y_extents: Vec<u64> = colon_delim_str_to_extents(ys);
    let z_extents: Vec<u64> = colon_delim_str_to_extents(zs);

    // Try to convert to origin-and-shape:
    let origin = Vector3 {
        x: x_extents[0],
        y: y_extents[0],
        z: z_extents[0],
    };
    let shape = Vector3 {
        x: x_extents[1] - x_extents[0],
        y: y_extents[1] - y_extents[0],
        z: z_extents[1] - z_extents[0],
    };
    let shape_dimension = (shape.z as usize, shape.y as usize, shape.x as usize);

    // TODO: Assert that shape is positive

    // Create a vector that'll carry the contents of the file:
    let mut vec: Vec<u8> = Vec::new();
    data.open().read_to_end(&mut vec).unwrap();

    // Decompress the data and rewrap it in an ndarray.
    // This is unsafe because the bytes are coming directly over the wire.
    let decompressed: Vec<u8> = unsafe { blosc::decompress_bytes(&vec[..]) }.unwrap();

    // Reshape the flat vec into a 3D ndarray:
    let array = Array::from_shape_vec(shape_dimension, decompressed).unwrap();

    // Perform the data-write:
    let fm = ChunkedBloscFileDataManager::new(
        format!(
            "upload/{collection}/{experiment}/{channel}/{res}",
            collection = collection,
            experiment = experiment,
            channel = channel,
            res = res
        ),
        Vector3 {
            x: 512,
            y: 512,
            z: 64,
        },
    );
    let result = fm.put_data(
        format!("bossdb://{}/{}/{}", collection, experiment, channel),
        res,
        origin,
        array,
    );

    status::Created(format!("{}", result), Some("{}".to_string()))
}

#[get("/")]
fn index() -> String {
    return format!("Hello world!");
}

#[catch(404)]
fn not_found(_req: &Request) { /* .. */
}

fn main() {
    rocket::custom(
        rocket::config::Config::build(rocket::config::Environment::Development)
            .port(8090)
            .unwrap(),
    )
    .mount(
        "/v1",
        routes![index, get_channel_metadata, upload, download],
    )
    .register(catchers![not_found])
    .launch();
}
