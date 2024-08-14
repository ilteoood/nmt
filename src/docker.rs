use bollard::image::BuildImageOptions;
use bollard::secret::BuildInfo;
use bollard::Docker;

use configurations::DockerConfigurations;
use futures_util::stream::StreamExt;

use std::io::Write;

mod configurations;

const DOCKERFILE_NAME: &str = "Dockerfile";

fn create_header(name: &str, size: usize) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_path(name).unwrap();
    header.set_size(size as u64);
    header.set_mode(0o755);
    header.set_cksum();

    header
}

fn create_dockerfile_header(dockerfile: &str) -> tar::Header {
    create_header(DOCKERFILE_NAME, dockerfile.len())
}

fn create_tar(dockerfile: &str) -> tar::Builder<Vec<u8>> {
    let mut tar = tar::Builder::new(Vec::new());
    tar.append(&create_dockerfile_header(dockerfile), dockerfile.as_bytes())
        .unwrap();

    tar
}

fn create_compressed_tar(dockerfile: &str) -> Vec<u8> {
    let tar = create_tar(dockerfile);

    let uncompressed = tar.into_inner().unwrap();

    let mut c = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    c.write_all(&uncompressed).unwrap();
    c.finish().unwrap()
}

fn create_dockerfile(configurations: &DockerConfigurations, history: History) -> String {
    format!(
        r##"FROM ilteoood/nmt as nmt_trimmer
    FROM {} as source_image
    COPY --from=nmt_trimmer ./cli .
    {}
    RUN ./cli && rm -f ./cli
    FROM scratch
    COPY --from=source_image / /
    {}
    {}
    {}"##,
        configurations.source_image,
        configurations.cli.to_dockerfile_env(),
        history.workdir,
        history.command,
        history.entry_point
    )
}

fn print_build_step(build_info: BuildInfo) {
    let content = build_info
        .status
        .or(build_info.stream)
        .unwrap_or(String::from(""));

    println!("{content}");
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

struct History {
    workdir: String,
    command: String,
    entry_point: String,
}

async fn retrieve_history(
    docker: &Docker,
    configurations: &DockerConfigurations,
) -> Result<History, bollard::errors::Error> {
    let history = docker
        .image_history(configurations.source_image.as_str())
        .await?;

    let mut entry_point: String = String::new();
    let mut command = String::new();
    let mut workdir = String::new();

    for history_item in history {
        match history_item.created_by.to_lowercase() {
            entry if entry_point.is_empty() && entry.starts_with("entrypoint") => {
                entry_point = entry.replace("[", "").replace("]", "")
            }
            cmd if command.is_empty() && cmd.starts_with("command") => command = cmd,
            wd if workdir.is_empty() && wd.starts_with("workdir") => workdir = wd,
            _ => {}
        }
    }

    Ok(History {
        workdir,
        command,
        entry_point,
    })
}

#[tokio::main]
async fn main() -> Result<(), bollard::errors::Error> {
    let configurations = DockerConfigurations::from_env();

    let docker = Docker::connect_with_socket_defaults().unwrap();

    let history = retrieve_history(&docker, &configurations).await?;

    let dockerfile = create_dockerfile(&configurations, history);

    let compressed_tar = create_compressed_tar(&dockerfile);

    let build_image_options = BuildImageOptions {
        dockerfile: DOCKERFILE_NAME,
        t: &configurations.destination_image,
        nocache: true,
        rm: true,
        pull: true,
        ..Default::default()
    };

    run_build(build_image_options, &docker, compressed_tar).await;

    Ok(())
}
