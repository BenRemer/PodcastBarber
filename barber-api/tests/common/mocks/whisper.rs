use testcontainers::{ContainerAsync, GenericImage, ImageExt};

use testcontainers::core::{IntoContainerPort, WaitFor};

use testcontainers::runners::AsyncRunner;

pub async fn start_sidecar() -> (ContainerAsync<GenericImage>, String) {
    let image = GenericImage::new("fedirz/faster-whisper-server", "latest-cuda")
        .with_wait_for(WaitFor::message_on_stderr("Application startup complete"))
        .with_exposed_port(8000.tcp())
        .with_env_var("WHISPER__MODEL", "tiny");

    let container = image.start().await.expect("Failed to start whisper");

    let port = container.get_host_port_ipv4(8000).await.unwrap();

    let url = format!("http://127.0.0.1:{}/v1", port);

    (container, url)
}
