use bollard::image::BuildImageOptions;
use bollard::secret::BuildInfo;
use bollard::Docker;

use futures_util::stream::StreamExt;
use nmt::configurations::DockerConfigurations;
use nmt::container_configurations::ContainerConfigurations;

use std::io::Write;

const DOCKERFILE_NAME: &str = "Dockerfile";

fn create_header(name: &str, size: usize) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_path(name).unwrap();
    header.set_size(size as u64);
    header.set_mode(0o755);
    header.set_cksum();

    header
}

fn create_tar(dockerfile: &str) -> tar::Builder<Vec<u8>> {
    let mut tar = tar::Builder::new(Vec::new());
    tar.append(
        &create_header(DOCKERFILE_NAME, dockerfile.len()),
        dockerfile.as_bytes(),
    )
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

fn create_dockerfile(
    configurations: &DockerConfigurations,
    container_configurations: ContainerConfigurations,
) -> String {
    format!(
        r##"FROM ilteoood/nmt as nmt_trimmer
    FROM {} as source_image
    COPY --from=nmt_trimmer ./cli .
    {}
    RUN ./cli && rm -f ./cli
    FROM scratch
    COPY --from=source_image / /
    {}"##,
        configurations.source_image,
        configurations.cli.to_dockerfile_env(),
        container_configurations.to_dockerfile(),
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

async fn pull_image(docker: &Docker, image_name: &str) {
    let dockerfile = format!("FROM {}", image_name);

    let compressed_tar = create_compressed_tar(dockerfile.as_str());

    run_build(
        BuildImageOptions {
            dockerfile: DOCKERFILE_NAME,
            nocache: true,
            rm: true,
            pull: true,
            ..Default::default()
        },
        docker,
        compressed_tar,
    )
    .await;
}

async fn retrieve_config(
    docker: &Docker,
    configurations: &DockerConfigurations,
) -> Result<ContainerConfigurations, bollard::errors::Error> {
    let source_image = configurations.source_image.as_str();
    pull_image(docker, source_image).await;

    let inspect = docker.inspect_image(source_image).await?;

    match inspect.config {
        Some(container_config) => Ok(ContainerConfigurations::from_container(container_config)),
        None => Ok(ContainerConfigurations::default()),
    }
}

#[tokio::main]
async fn main() -> Result<(), bollard::errors::Error> {
    let configurations = DockerConfigurations::new();

    let docker = Docker::connect_with_socket_defaults().unwrap();

    let container_config = retrieve_config(&docker, &configurations).await?;

    let dockerfile = create_dockerfile(&configurations, container_config);

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

#[cfg(test)]
mod history_tests {
    use nmt::configurations::CliConfigurations;

    use super::*;

    #[tokio::test]
    async fn test_empty_container_configurations() {
        let container_configurations = retrieve_config(
            &Docker::connect_with_socket_defaults().unwrap(),
            &DockerConfigurations::new(),
        )
        .await
        .unwrap();

        assert_eq!(
            container_configurations,
            ContainerConfigurations {
                workdir: Some(String::from("WORKDIR /")),
                command: Some(String::from("CMD /hello")),
                entry_point: None,
                health_check: None,
                user: None,
                env: Some(String::from(
                    "ENV PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
                )),
            }
        );
    }

    #[tokio::test]
    async fn test_history() {
        let container_configurations = retrieve_config(
            &Docker::connect_with_socket_defaults().unwrap(),
            &DockerConfigurations {
                source_image: String::from("ilteoood/xdcc-mule"),
                destination_image: String::from("ilteoood/xdcc-mule"),
                cli: CliConfigurations::new(),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            container_configurations,
            ContainerConfigurations {
                workdir: Some(String::from("WORKDIR /app")),
                command: None,
                entry_point: Some(String::from("ENTRYPOINT node index.js")),
                health_check: None,
                user: None,
                env: Some(String::from("ENV PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\nENV NODE_VERSION=20.17.0\nENV YARN_VERSION=1.22.22")),
            }
        );
    }
}
