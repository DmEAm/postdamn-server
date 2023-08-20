use std::convert::Infallible;
use std::env;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use error_chain::error_chain;
use tch::{Cuda, Device, Tensor, vision::resnet};
use tch::nn::VarStore;
use tracing;

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
     }
}
async fn cached_weights() -> std::result::Result<VarStore, Infallible> {
    let weights_url = "https://download.pytorch.org/models/resnet50-0676ba61.pth";
    let response = reqwest::get(weights_url).await?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.pth");

        let fname = env::current_dir().expect("must exist").join(fname.clone());

        println!("file to download: '{:?}'", fname.file_name());
        println!("will be located under: '{:?}'", fname);
        File::create(fname)?
    };
    let content = response.text().await?;
    copy(&mut content.as_bytes(), &mut dest)?;

    VarStore::try_from(dest.)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::debug!("Cuda is available {}", Cuda::is_available());

    let weights = cached_weights().await?;
    let net = resnet::resnet50(&weights.root(), 10);
    Ok(())
}
