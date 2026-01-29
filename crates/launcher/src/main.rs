use iced::widget::{button, column, container, row, text, text_input};
use iced::{Center, Element, Fill, Task};
use std::path::PathBuf;

mod config;
use config::Config;

fn main() -> iced::Result {
    println!("===========================================");
    println!("    Ragnoria Launcher v0.1.0");
    println!("    RO2 Custom Server Launcher");
    println!("===========================================");
    println!("Starting GUI application...\n");
    
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
        println!("Loading configuration...");
        let config = Config::load().unwrap_or_default();
        
        println!("Config loaded:");
        println!("  Server IP: {}", config.server.ip);
        println!("  Server Port: {}", config.server.port);
        println!("  Game Path: {}", if config.game_path.is_empty() { "(not set)" } else { &config.game_path });
        println!();

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
                println!("Server IP changed to: {}", ip);
                self.server_ip = ip;
                Task::none()
            }
            Message::ServerPortChanged(port) => {
                println!("Server port changed to: {}", port);
                self.server_port = port;
                Task::none()
            }
            Message::GamePathChanged(path) => {
                println!("Game path changed to: {}", path);
                self.game_path = path;
                Task::none()
            }
            Message::LaunchGame => {
                println!("\n=== LAUNCH GAME BUTTON CLICKED ===");
                println!("Server IP: {}", self.server_ip);
                println!("Server Port: {}", self.server_port);
                println!("Game Path: {}", self.game_path);
                self.launch_game();
                println!("=== LAUNCH ATTEMPT COMPLETE ===\n");
                Task::none()
            }
            Message::BrowseGamePath => {
                println!("Browse button clicked - opening file dialog...");
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Rag2 Executable", &["exe"])
                            .set_title("Select Rag2.exe")
                            .pick_file()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    Message::GamePathSelected,
                )
            }
            Message::GamePathSelected(path) => {
                if let Some(path) = path {
                    println!("File selected: {:?}", path);
                    self.game_path = path.to_string_lossy().to_string();
                    self.status_message = format!("Selected: {}", self.game_path);
                } else {
                    println!("File selection cancelled");
                    self.status_message = String::from("File selection cancelled");
                }
                Task::none()
            }
        }
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
        println!("Validating inputs...");
        
        // Validate inputs
        if self.server_ip.trim().is_empty() {
            let msg = "Error: Server IP is required";
            println!("VALIDATION ERROR: {}", msg);
            self.status_message = String::from(msg);
            return;
        }
        println!("✓ Server IP is valid: {}", self.server_ip);

        let port = match self.server_port.parse::<u16>() {
            Ok(p) => {
                println!("✓ Server port is valid: {}", p);
                p
            }
            Err(e) => {
                let msg = format!("Error: Invalid port number - {}", e);
                println!("VALIDATION ERROR: {}", msg);
                self.status_message = msg;
                return;
            }
        };

        if self.game_path.trim().is_empty() {
            let msg = "Error: Game path is required";
            println!("VALIDATION ERROR: {}", msg);
            self.status_message = String::from(msg);
            return;
        }
        println!("✓ Game path provided: {}", self.game_path);

        let game_path = PathBuf::from(&self.game_path);
        if !game_path.exists() {
            let msg = format!("Error: Game executable not found at {}", self.game_path);
            println!("VALIDATION ERROR: {}", msg);
            self.status_message = msg;
            return;
        }
        println!("✓ Game executable exists");

        // Save config before launching
        println!("Saving configuration...");
        self.config.server.ip = self.server_ip.clone();
        self.config.server.port = port;
        self.config.game_path = self.game_path.clone();

        if let Err(e) = self.config.save() {
            eprintln!("Warning: Failed to save config: {}", e);
        } else {
            println!("✓ Configuration saved");
        }

        // Launch the game with parameters
        println!("Attempting to launch game...");
        match self.launch_game_process() {
            Ok(_) => {
                let msg = format!("Game launched! Connecting to {}:{}", self.server_ip, port);
                println!("✓ SUCCESS: {}", msg);
                self.status_message = msg;
            }
            Err(e) => {
                let msg = format!("Error launching game: {}", e);
                println!("✗ LAUNCH ERROR: {}", msg);
                self.status_message = msg;
            }
        }
    }

    fn launch_game_process(&self) -> anyhow::Result<()> {
        use std::process::Command;

        let game_path = PathBuf::from(&self.game_path);
        
        // Get the SHIPPING directory (parent of Rag2.exe)
        let shipping_dir = game_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid game path"))?;
        
        // Get the root game directory (parent of SHIPPING, where DLLs are)
        let game_root_dir = shipping_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid game directory structure"))?;

        println!("Game executable: {:?}", game_path);
        println!("SHIPPING directory: {:?}", shipping_dir);
        println!("Game root directory (with DLLs): {:?}", game_root_dir);
        println!("Working directory will be set to: {:?}", game_root_dir);
        println!();
        println!("NOTE: Based on Ghidra analysis, the game calls:");
        println!("  - ParseGameLaunchArguments() to parse command line");
        println!("  - GetGameServerIPFromCommandLine() to extract server IP");
        println!("  - ValidateGameLaunchParameters() to check for '-FromLauncher' flag");
        println!();
        println!("CRITICAL: Game REQUIRES '-FromLauncher' flag in command line!");
        println!("Without it, shows Korean error dialogs about Updater.exe");
        println!();

        // Based on Ghidra analysis of Rag2.exe:
        // CRITICAL DISCOVERY: ValidateGameLaunchParameters() checks for "-FromLauncher" flag!
        // If this flag is NOT present, the game shows two error dialogs:
        //   1. Korean error about "Updater" needing to launch the game
        //   2. Check for ../Updater.exe and shows error if not found
        //
        // The decompiled code shows:
        //   if (!strcmp(commandLine, "-FromLauncher")) return; // Success!
        //   else show_error("Updater") and check for ../Updater.exe
        //
        // SOLUTION: Always include "-FromLauncher" in command line!
        
        // Try different parameter formats (all include -FromLauncher)
        let commands_to_try = vec![
            // Option 0: Just -FromLauncher flag (RECOMMENDED - Ghidra analysis)
            vec![
                String::from("-FromLauncher"),
            ],
            
            // Option 1: -FromLauncher + server IP and port
            vec![
                String::from("-FromLauncher"),
                self.server_ip.clone(),
                self.server_port.clone(),
            ],
            
            // Option 2: -FromLauncher + /IP and /PORT flags
            vec![
                String::from("-FromLauncher"),
                format!("/IP={}", self.server_ip),
                format!("/PORT={}", self.server_port),
            ],
            
            // Option 3: -FromLauncher + combined IP:PORT
            vec![
                String::from("-FromLauncher"),
                format!("{}:{}", self.server_ip, self.server_port),
            ],
            
            // Option 4: NO parameters (will show Updater error!)
            vec![],
        ];

        // Use Option 0 by default (just -FromLauncher flag)
        // You can change this to test different parameter combinations:
        // 0 = -FromLauncher only (RECOMMENDED - bypasses Updater check)
        // 1 = -FromLauncher + IP PORT
        // 2 = -FromLauncher + /IP=x.x.x.x /PORT=xxxx
        // 3 = -FromLauncher + IP:PORT combined
        // 4 = No parameters (will show Updater error for testing)
        let option_index = std::env::var("LAUNCH_OPTION")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        
        let args = commands_to_try.get(option_index).unwrap_or(&commands_to_try[0]);

        println!("Using launch option {}: {:?}", option_index, args);
        println!();
        println!("To try different options, set LAUNCH_OPTION environment variable:");
        println!("  LAUNCH_OPTION=0  - -FromLauncher only (RECOMMENDED - default)");
        println!("  LAUNCH_OPTION=1  - -FromLauncher + IP PORT");
        println!("  LAUNCH_OPTION=2  - -FromLauncher + /IP=x.x.x.x /PORT=xxxx");
        println!("  LAUNCH_OPTION=3  - -FromLauncher + IP:PORT combined");
        println!("  LAUNCH_OPTION=4  - No parameters (ERROR TEST - will show Updater error)");
        println!();

        #[cfg(target_os = "windows")]
        {
            println!("Platform: Windows (native execution)");
            let result = Command::new(&game_path)
                .args(args)
                .current_dir(game_root_dir)
                .spawn();
            
            match &result {
                Ok(child) => println!("✓ Process spawned successfully! PID: {:?}", child.id()),
                Err(e) => println!("✗ Failed to spawn process: {}", e),
            }
            
            result?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("Platform: Linux/Unix (Wine execution)");
            let result = Command::new("wine")
                .arg(&game_path)
                .args(args)
                .current_dir(game_root_dir)
                .spawn();
            
            match &result {
                Ok(child) => println!("✓ Process spawned successfully! PID: {:?}", child.id()),
                Err(e) => println!("✗ Failed to spawn process: {}", e),
            }
            
            result?;
        }

        Ok(())
    }
}
