use iced::widget::{button, column, container, row, text, text_input};
use iced::{Center, Element, Fill, Task};
use std::path::PathBuf;

mod config;
use config::Config;

fn main() -> iced::Result {
    iced::application("Ragnoria Launcher", Launcher::update, Launcher::view)
        .window_size((600.0, 400.0))
        .run_with(Launcher::new)
}

#[derive(Debug, Clone)]
enum Message {
    ServerIpChanged(String),
    ServerPortChanged(String),
    GamePathChanged(String),
    LaunchGame,
    BrowseGamePath,
    GamePathSelected(Option<PathBuf>),
}

struct Launcher {
    server_ip: String,
    server_port: String,
    game_path: String,
    status_message: String,
    config: Config,
}

impl Launcher {
    fn new() -> (Self, Task<Message>) {
        let config = Config::load().unwrap_or_default();

        let launcher = Self {
            server_ip: config.server.ip.clone(),
            server_port: config.server.port.to_string(),
            game_path: config.game_path.clone(),
            status_message: String::from("Ready to launch"),
            config,
        };

        (launcher, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerIpChanged(ip) => {
                self.server_ip = ip;
                Task::none()
            }
            Message::ServerPortChanged(port) => {
                self.server_port = port;
                Task::none()
            }
            Message::GamePathChanged(path) => {
                self.game_path = path;
                Task::none()
            }
            Message::LaunchGame => {
                self.launch_game();
                Task::none()
            }
            Message::BrowseGamePath => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Rag2 Executable", &["exe"])
                        .set_title("Select Rag2.exe")
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                Message::GamePathSelected,
            ),
            Message::GamePathSelected(path) => {
                if let Some(path) = path {
                    self.game_path = path.to_string_lossy().to_string();
                    self.status_message = format!("Selected: {}", self.game_path);
                } else {
                    self.status_message = String::from("File selection cancelled");
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let title = text("Ragnoria Launcher").size(28).width(Fill);
        let subtitle = text("Connect to custom RO2 server").size(14).width(Fill);

        let server_ip_row = row![
            text("Server IP:").width(120),
            text_input("127.0.0.1", &self.server_ip)
                .on_input(Message::ServerIpChanged)
                .padding(8)
                .width(Fill)
        ]
        .spacing(10)
        .width(Fill);

        let server_port_row = row![
            text("Server Port:").width(120),
            text_input("7101", &self.server_port)
                .on_input(Message::ServerPortChanged)
                .padding(8)
                .width(Fill)
        ]
        .spacing(10)
        .width(Fill);

        let game_path_row = row![
            text("Game Path:").width(120),
            text_input("Path to Rag2.exe", &self.game_path)
                .on_input(Message::GamePathChanged)
                .padding(8)
                .width(Fill),
            button(text("Browse"))
                .on_press(Message::BrowseGamePath)
                .padding(8)
        ]
        .spacing(10)
        .width(Fill);

        let launch_button = button(text("Launch Game").width(Fill).align_x(Center))
            .on_press(Message::LaunchGame)
            .padding(12)
            .width(200);

        let status = text(&self.status_message).size(12).width(Fill);

        let content = column![
            title,
            subtitle,
            server_ip_row,
            server_port_row,
            game_path_row,
            launch_button,
            status,
        ]
        .spacing(15)
        .padding(30)
        .width(Fill);

        container(content)
            .width(Fill)
            .height(Fill)
            .padding(20)
            .into()
    }

    fn launch_game(&mut self) {
        // Validate inputs
        if self.server_ip.trim().is_empty() {
            self.status_message = String::from("Error: Server IP is required");
            return;
        }

        let port = match self.server_port.parse::<u16>() {
            Ok(p) => p,
            Err(e) => {
                self.status_message = format!("Error: Invalid port - {}", e);
                return;
            }
        };

        if self.game_path.trim().is_empty() {
            self.status_message = String::from("Error: Game path is required");
            return;
        }

        let game_path = PathBuf::from(&self.game_path);
        if !game_path.exists() {
            self.status_message = format!("Error: Game not found at {}", self.game_path);
            return;
        }

        // Save config
        self.config.server.ip = self.server_ip.clone();
        self.config.server.port = port;
        self.config.game_path = self.game_path.clone();
        let _ = self.config.save();

        // Launch game
        match self.launch_game_process(&game_path, &self.server_ip) {
            Ok(_) => {
                self.status_message =
                    format!("Game launched! Connecting to {}:{}", self.server_ip, port);
            }
            Err(e) => {
                self.status_message = format!("Error launching game: {}", e);
            }
        }
    }

    fn launch_game_process(&self, game_path: &PathBuf, server_ip: &str) -> anyhow::Result<()> {
        use std::process::Command;

        // Get directories
        let shipping_dir = game_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid game path"))?;

        let game_root_dir = shipping_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid game directory structure"))?;

        // Build command line arguments
        // Format: /FROM=-FromLauncher /IP=127.0.0.1
        let args = vec![
            String::from("/FROM=-FromLauncher"),
            format!("/IP={}", server_ip),
        ];

        // Launch game
        #[cfg(target_os = "windows")]
        {
            Command::new(game_path)
                .args(&args)
                .current_dir(game_root_dir)
                .spawn()?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            Command::new("wine")
                .arg(game_path)
                .args(&args)
                .current_dir(game_root_dir)
                .spawn()?;
        }

        Ok(())
    }
}
