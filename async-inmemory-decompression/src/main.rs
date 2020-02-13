use anyhow::{bail, Result};
use async_compression::futures::bufread::GzipDecoder;
use async_std::io::ReadExt;
use async_tar::Archive;

/// This currently throws the following error:
/// ```
/// future cannot be sent between threads safely
/// future returned by `extract_file` is not `Send`
/// help: within `async_tar::archive::ArchiveInner<async_compression::futures::bufread::GzipDecoder<&[u8]>>`, the trait `std::marker::Sync` is not implemented for `std::cell::Cell<u64>`
/// note: the return type of a function must have a statically known sizerustc
/// main.rs(9, 6): future returned by `extract_file` is not `Send`
/// main.rs(13, 13): has type `async_tar::archive::Archive<async_compression::futures::bufread::GzipDecoder<&[u8]>>`
/// main.rs(39, 9): await occurs here, with `mut archive` maybe used later
/// main.rs(42, 5): `mut archive` is later dropped here
/// ```
fn extract_file(
    blob: Vec<u8>,
    filename: String,
) -> impl std::future::Future<Output = Result<String>> + Send {
    use async_std::stream::StreamExt;

    async move {
        let mut archive = Archive::new(GzipDecoder::new(blob.as_slice()));
        let mut file =
            match async_std::stream::StreamExt::filter_map(
                archive.entries()?,
                |entry| match entry {
                    Ok(file) => Some(file),
                    Err(err) => {
                        eprintln!("failed to read archive entry: {}", err);
                        None
                    }
                },
            )
            .find(|file| match file.header().path() {
                Ok(path) => &(*path) == async_std::path::Path::new(&filename),
                Err(err) => {
                    eprintln!("failed to read file header: {}", err);
                    false
                }
            })
            .await
            {
                Some(file) => file,
                None => bail!(format!("'{}' not found", filename)),
            };

        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        Ok(contents)
    }
}
#[tokio::main]
async fn main() {
    let string_future = extract_file("invalid".into(), "invalid".into()).await;

    std::thread::spawn(move || {
        println!("{}", string_future.unwrap());
    });
}
