use bollard::image::BuildImageOptions;
use bollard::Docker;

use futures_util::stream::StreamExt;

use std::io::Write;

fn create_header(dockerfile: &str) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_path("Dockerfile").unwrap();
    header.set_size(dockerfile.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();

    header
}

fn create_tar(dockerfile: &str) -> tar::Builder<Vec<u8>> {
    let header = create_header(&dockerfile);
    let mut tar = tar::Builder::new(Vec::new());
    tar.append(&header, dockerfile.as_bytes()).unwrap();

    tar
}

fn create_compressed_tar(dockerfile: &str) -> Vec<u8> {
    let tar = create_tar(&dockerfile);

    let uncompressed = tar.into_inner().unwrap();

    let mut c = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    c.write_all(&uncompressed).unwrap();
    c.finish().unwrap().into()
}

fn create_dockerfile(source_image: &str) -> String {
    format!(
        "FROM {source_image}
RUN touch buildkit-bollard.txt"
    )
}

async fn run_build(build_image_options: BuildImageOptions<&str>, docker: &Docker, compressed_tar: Vec<u8>) {
    let mut image_build_stream =
        docker.build_image(build_image_options, None, Some(compressed_tar.into()));

    while let Some(msg) = image_build_stream.next().await {
        let msg = msg.unwrap();

        let content = msg
            .status
            .or_else(|| msg.stream)
            .or_else(|| Some(String::from("")))
            .unwrap();

        print!("{content}");
    }
}

#[tokio::main]
async fn main() {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let dockerfile = create_dockerfile("alpine");

    let compressed_tar = create_compressed_tar(&dockerfile);

    let build_image_options = BuildImageOptions {
        dockerfile: "Dockerfile",
        pull: true,
        ..Default::default()
    };

    run_build(build_image_options, &docker, compressed_tar).await;
}
