use bollard::secret::ContainerConfig;

/// Container configurations
#[derive(Debug, PartialEq, Default)]
pub struct ContainerConfigurations {
    /// WORKDIR instruction
    pub workdir: Option<String>,
    /// CMD instruction
    pub command: Option<String>,
    /// ENTRYPOINT instruction
    pub entry_point: Option<String>,
    /// HEALTHCHECK instruction
    pub health_check: Option<String>,
    /// USER instruction
    pub user: Option<String>,
    /// ENV instruction
    pub env: Option<String>,
}

impl ContainerConfigurations {
    /// Creates a new `ContainerConfigurations` from a `ContainerConfig`
    pub fn from_container(container_config: ContainerConfig) -> ContainerConfigurations {
        ContainerConfigurations {
            workdir: container_config
                .working_dir
                .map(|workdir| format!("WORKDIR {workdir}")),
            command: container_config
                .cmd
                .map(|command| format!("CMD {command:?}")),
            entry_point: container_config
                .entrypoint
                .map(|entry_point| format!("ENTRYPOINT {entry_point:?}")),
            user: match container_config.user {
                Some(ref user) if !user.is_empty() => Some(format!("USER {user}")),
                _ => None,
            },
            health_check: match container_config.healthcheck {
                Some(health_check_config) => {
                    let mut health_check = String::from("HEALTHCHECK");

                    if let Some(interval) = health_check_config.interval {
                        health_check = format!("{health_check} --interval={interval}ms");
                    }

                    if let Some(timeout) = health_check_config.timeout {
                        health_check = format!("{health_check} --timeout={timeout}ms");
                    }

                    if let Some(start_period) = health_check_config.start_period {
                        health_check = format!("{health_check} --start-period={start_period}ms");
                    }

                    if let Some(start_interval) = health_check_config.start_interval {
                        health_check =
                            format!("{health_check} --start-interval={start_interval}ms");
                    }

                    if let Some(retries) = health_check_config.retries {
                        health_check = format!("{health_check} --retries={retries}");
                    }

                    if let Some(test) = health_check_config.test {
                        health_check = format!(
                            "{} {}",
                            health_check,
                            test.iter()
                                .map(|item| if item == "CMD-SHELL" { "CMD" } else { item })
                                .collect::<Vec<&str>>()
                                .join(" ")
                        );
                    }

                    Some(health_check)
                }
                None => None,
            },
            env: match container_config.env {
                Some(container_env) => {
                    let envs = container_env
                        .iter()
                        .filter_map(|env_value| {
                            env_value
                                .split_once('=')
                                .map(|(key, value)| format!("ENV {key}={value}"))
                        })
                        .collect::<Vec<_>>();

                    Some(envs.join("\n"))
                }
                None => None,
            },
        }
    }

    /// Converts the container configurations to a Dockerfile
    pub fn to_dockerfile(&self) -> String {
        [
            self.workdir.clone(),
            self.command.clone(),
            self.entry_point.clone(),
            self.user.clone(),
            self.env.clone(),
            self.health_check.clone(),
        ]
        .iter()
        .filter_map(std::clone::Clone::clone)
        .collect::<Vec<String>>()
        .join("\n")
    }
}
