use bollard::image::BuildImageOptions;
use bollard::secret::BuildInfo;
use bollard::Docker;

use futures_util::stream::StreamExt;

use std::io::Write;

fn create_header(name: &str, size: usize) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_path(name).unwrap();
    header.set_size(size as u64);
    header.set_mode(0o755);
    header.set_cksum();

    header
}

fn create_dockerfile_header(dockerfile: &str) -> tar::Header {
    create_header("Dockerfile", dockerfile.len())
}

fn create_tar(dockerfile: &str) -> tar::Builder<Vec<u8>> {
    let mut tar = tar::Builder::new(Vec::new());
    tar.append(
        &create_dockerfile_header(&dockerfile),
        dockerfile.as_bytes(),
    )
    .unwrap();

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
        "FROM ilteoood/nmt as trimmer
FROM {source_image}
COPY --from=trimmer ./cli .
RUN ./cli && rm -f ./cli"
    )
}

fn print_build_step(build_info: BuildInfo) {
    let content = build_info
        .status
        .or_else(|| build_info.stream)
        .or_else(|| Some(String::from("")))
        .unwrap();

    print!("{content}");
}

async fn run_build(
    build_image_options: BuildImageOptions<&str>,
    docker: &Docker,
    compressed_tar: Vec<u8>,
) {
    let mut image_build_stream =
        docker.build_image(build_image_options, None, Some(compressed_tar.into()));

    while let Some(msg) = image_build_stream.next().await {
        match msg {
            Ok(build_info) => print_build_step(build_info),
            Err(err) => println!("Error while building image: {err}"),
        }
    }
}

#[tokio::main]
async fn main() {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let dockerfile = create_dockerfile("debian");

    let compressed_tar = create_compressed_tar(&dockerfile);

    let build_image_options = BuildImageOptions {
        dockerfile: "Dockerfile",
        nocache: true,
        rm: true,
        pull: true,
        ..Default::default()
    };

    run_build(build_image_options, &docker, compressed_tar).await;
}
