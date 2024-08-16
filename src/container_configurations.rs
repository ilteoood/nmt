use bollard::secret::ContainerConfig;

#[derive(Debug, PartialEq)]
pub struct ContainerConfigurations {
    pub workdir: Option<String>,
    pub command: Option<String>,
    pub entry_point: Option<String>,
    pub health_check: Option<String>,
    pub user: Option<String>,
    pub env: Option<String>,
}

impl ContainerConfigurations {
    pub fn new() -> ContainerConfigurations {
        ContainerConfigurations {
            workdir: None,
            command: None,
            entry_point: None,
            health_check: None,
            user: None,
            env: None,
        }
    }

    pub fn from_container(container_config: ContainerConfig) -> ContainerConfigurations {
        ContainerConfigurations {
            workdir: match container_config.working_dir {
                Some(workdir) => Some(String::from(format!("WORKDIR {}", workdir))),
                None => None,
            },
            command: match container_config.cmd {
                Some(command) => Some(String::from(format!("CMD {}", command.join(" ")))),
                None => None,
            },
            entry_point: match container_config.entrypoint {
                Some(entry_point) => Some(String::from(format!(
                    "ENTRYPOINT {}",
                    entry_point.join(" ")
                ))),
                None => None,
            },
            user: match container_config.user {
                Some(ref user) if !user.is_empty() => Some(String::from(format!("USER {}", user))),
                _ => None,
            },
            health_check: match container_config.healthcheck {
                Some(health_check_config) => {
                    let mut health_check = String::from("HEALTHCHECK");

                    if let Some(interval) = health_check_config.interval {
                        health_check = format!("{} --interval={}", health_check, interval);
                    }

                    if let Some(timeout) = health_check_config.timeout {
                        health_check = format!("{} --timeout={}", health_check, timeout);
                    }

                    if let Some(start_period) = health_check_config.start_period {
                        health_check = format!("{} --start-period={}", health_check, start_period);
                    }

                    if let Some(start_interval) = health_check_config.start_interval {
                        health_check = format!("{} --retries={}", health_check, start_interval);
                    }

                    if let Some(retries) = health_check_config.retries {
                        health_check = format!("{} --retries={}", health_check, retries);
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
                                .map(|(key, value)| format!("ENV {}={}", key, value))
                        })
                        .collect::<Vec<_>>();

                    Some(envs.join("\n"))
                }
                None => None,
            },
        }
    }

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
        .filter_map(|x| x.as_ref())
        .map(|x| x.clone())
        .collect::<Vec<String>>()
        .join("\n")
    }
}
