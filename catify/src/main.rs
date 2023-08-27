use std::env;
use std::fs::{File, OpenOptions};
use std::io::copy;

use error_chain::error_chain;
use reqwest::Url;
use tch::{nn::ModuleT, Cuda, Device, vision::resnet, TrainableCModule};
use tch::Kind::Float;
use tch::nn::VarStore;
use tch::vision::imagenet;
use tracing;
use tracing::debug;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

error_chain! {
     foreign_links {
         Torch(tch::TchError);
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
     }
}
async fn trained_model() -> Result<VarStore> {
    let weights_url =
        Url::parse("https://download.pytorch.org/models/resnet50-0676ba61.pth").expect("valid url");

    let file_name = weights_url
        .path_segments()
        .and_then(|segments| segments.last())
        .unwrap_or("tmp.pth");
    let file_name = env::current_dir()
        .expect("must exist")
        .join("models")
        .join(file_name);

    let response = reqwest::get(weights_url).await?;

    let mut dest = {
        if file_name.exists() {
            debug!("model file exist: '{:?}'", file_name.file_name().unwrap());
            OpenOptions::new()
                .read(true)
                .write(false)
                .truncate(false)
                .open(&file_name)?
        } else {
            debug!("file to download: '{:?}'", file_name.file_name().unwrap());
            debug!("will be located under: '{:?}'", &file_name);
            let mut dest = File::create(&file_name)?;
            let content = response.text().await?;
            copy(&mut content.as_bytes(), &mut dest)?;
            dest
        }
    };

    let vs = VarStore::new(Device::cuda_if_available());
    Ok(vs)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "catify=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("Cuda is available {}", Cuda::is_available());

    let weights = trained_model().await?;
    let model = resnet::resnet50(&weights.root(), 10);

    let image = imagenet::load_image_and_resize("tests/1.jpeg".to_owned(), 640, 640)?;
    let output = model
        .forward_t(&image.unsqueeze(0), false)
        .softmax(-1, Float);
    debug!("{}", output);
    Ok(())
}
