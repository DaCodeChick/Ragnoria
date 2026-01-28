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
            }
            Message::ServerPortChanged(port) => {
                self.server_port = port;
            }
            Message::GamePathChanged(path) => {
                self.game_path = path;
            }
            Message::LaunchGame => {
                self.launch_game();
            }
            Message::BrowseGamePath => {
                self.status_message = String::from(
                    "File browser not yet implemented. Please type the full path to Rag2.exe",
                );
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let title = text("Ragnoria Launcher").size(28).width(Fill);

        let subtitle = text("Connect to custom RO2 server").size(14).width(Fill);

        let server_ip_label = text("Server IP:").width(120);
        let server_ip_input = text_input("127.0.0.1", &self.server_ip)
            .on_input(Message::ServerIpChanged)
            .padding(8)
            .width(Fill);

        let server_ip_row = row![server_ip_label, server_ip_input]
            .spacing(10)
            .width(Fill);

        let server_port_label = text("Server Port:").width(120);
        let server_port_input = text_input("7101", &self.server_port)
            .on_input(Message::ServerPortChanged)
            .padding(8)
            .width(Fill);

        let server_port_row = row![server_port_label, server_port_input]
            .spacing(10)
            .width(Fill);

        let game_path_label = text("Game Path:").width(120);
        let game_path_input = text_input("Path to Rag2.exe", &self.game_path)
            .on_input(Message::GamePathChanged)
            .padding(8)
            .width(Fill);
        let browse_button = button(text("Browse"))
            .on_press(Message::BrowseGamePath)
            .padding(8);

        let game_path_row = row![game_path_label, game_path_input, browse_button]
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
            Err(_) => {
                self.status_message = String::from("Error: Invalid port number");
                return;
            }
        };

        if self.game_path.trim().is_empty() {
            self.status_message = String::from("Error: Game path is required");
            return;
        }

        let game_path = PathBuf::from(&self.game_path);
        if !game_path.exists() {
            self.status_message = format!("Error: Game executable not found at {}", self.game_path);
            return;
        }

        // Save config before launching
        self.config.server.ip = self.server_ip.clone();
        self.config.server.port = port;
        self.config.game_path = self.game_path.clone();

        if let Err(e) = self.config.save() {
            eprintln!("Warning: Failed to save config: {}", e);
        }

        // Launch the game with parameters
        match self.launch_game_process() {
            Ok(_) => {
                self.status_message =
                    format!("Game launched! Connecting to {}:{}", self.server_ip, port);
            }
            Err(e) => {
                self.status_message = format!("Error launching game: {}", e);
            }
        }
    }

    fn launch_game_process(&self) -> anyhow::Result<()> {
        use std::process::Command;

        let game_path = PathBuf::from(&self.game_path);
        let game_dir = game_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid game path"))?;

        // Try multiple parameter formats based on our analysis
        // Format from RO2Client.exe: /FROM=-FromUpdater /STARTER=2 [additional_params]
        // We need to test what parameters Rag2.exe accepts for server IP/port

        let commands_to_try = vec![
            // Option 1: Custom parameters (most likely)
            vec![
                format!("/FROM=-Ragnoria"),
                format!("/STARTER=2"),
                format!("/IP={}", self.server_ip),
                format!("/PORT={}", self.server_port),
            ],
            // Option 2: Combined server parameter
            vec![
                format!("/FROM=-Ragnoria"),
                format!("/STARTER=2"),
                format!("/SERVER={}:{}", self.server_ip, self.server_port),
            ],
            // Option 3: LoginServer parameter
            vec![
                format!("/FROM=-Ragnoria"),
                format!("/STARTER=2"),
                format!("/LOGINSERVER={}:{}", self.server_ip, self.server_port),
            ],
        ];

        // For now, try the first option
        let args = &commands_to_try[0];

        println!("Launching game: {:?} with args: {:?}", game_path, args);

        #[cfg(target_os = "windows")]
        {
            Command::new(&game_path)
                .args(args)
                .current_dir(game_dir)
                .spawn()?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Linux, try with Wine
            Command::new("wine")
                .arg(&game_path)
                .args(args)
                .current_dir(game_dir)
                .spawn()?;
        }

        Ok(())
    }
}
