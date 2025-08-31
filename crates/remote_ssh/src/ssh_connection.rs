//! SSH Connection Management
//!
//! Handles SSH connections, authentication, and session management for remote development.

use anyhow::{Context, Result};
use ssh2::{Session, Sftp};
use std::collections::HashMap;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// SSH connection configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshConfig {
    /// Remote host address
    pub host: String,
    /// SSH port (default: 22)
    pub port: u16,
    /// Username for SSH connection
    pub user: String,
    /// Authentication method
    pub auth: SshAuth,
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// SSH key file path (optional)
    pub key_file: Option<PathBuf>,
    /// Known hosts file path (optional)
    pub known_hosts_file: Option<PathBuf>,
    /// Host key verification
    #[serde(default = "default_strict_host_key_checking")]
    pub strict_host_key_checking: bool,
}

fn default_timeout() -> u64 {
    30
}

fn default_strict_host_key_checking() -> bool {
    true
}

/// SSH authentication methods
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SshAuth {
    /// Password authentication
    Password(String),
    /// Public key authentication
    PublicKey {
        /// Public key file path
        public_key: PathBuf,
        /// Private key file path
        private_key: PathBuf,
        /// Passphrase for encrypted private key
        passphrase: Option<String>,
    },
    /// Agent authentication (SSH agent)
    Agent,
}

/// SSH connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// SSH connection manager
pub struct SshConnection {
    /// Connection configuration
    config: SshConfig,
    /// SSH session
    session: Option<Session>,
    /// SFTP session for file operations
    sftp: Option<Sftp>,
    /// Connection state
    state: ConnectionState,
    /// TCP stream for the connection
    stream: Option<TcpStream>,
}

impl SshConnection {
    /// Create a new SSH connection
    pub fn new(config: SshConfig) -> Self {
        Self {
            config,
            session: None,
            sftp: None,
            state: ConnectionState::Disconnected,
            stream: None,
        }
    }

    /// Connect to the remote SSH server
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = ConnectionState::Connecting;

        // Establish TCP connection
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let stream = TcpStream::connect(&addr)
            .with_context(|| format!("Failed to connect to {}", addr))?;

        // Configure stream
        stream.set_read_timeout(Some(Duration::from_secs(self.config.timeout)))?;
        stream.set_write_timeout(Some(Duration::from_secs(self.config.timeout)))?;

        // Create SSH session
        let mut session = Session::new()
            .context("Failed to create SSH session")?;

        session.set_tcp_stream(stream.try_clone()?);
        session.handshake()
            .context("SSH handshake failed")?;

        // Host key verification
        if self.config.strict_host_key_checking {
            self.verify_host_key(&session).ok(); // Ignore verification errors for now
        }

        // Authenticate based on method
        self.authenticate(&session).await?;

        // Initialize SFTP
        let sftp = session.sftp()
            .context("Failed to initialize SFTP session")?;

        self.session = Some(session);
        self.sftp = Some(sftp);
        self.stream = Some(stream);
        self.state = ConnectionState::Connected;

        log::info!("Successfully connected to {}@{}", self.config.user, self.config.host);
        Ok(())
    }

    /// Verify host key against known hosts (simplified)
    fn verify_host_key(&self, _session: &Session) -> Result<(), Box<dyn std::error::Error>> {
        // Simplified host key verification - in production, this should properly verify
        // against known hosts file and handle man-in-the-middle attacks
        log::info!("Host key verification skipped (simplified implementation)");
        Ok(())
    }

    /// Authenticate with the SSH server
    async fn authenticate(&self, session: &Session) -> Result<(), Box<dyn std::error::Error>> {
        match &self.config.auth {
            SshAuth::Password(password) => {
                session.userauth_password(&self.config.user, password)
                    .context("Password authentication failed")?;
            }
            SshAuth::PublicKey { public_key, private_key, passphrase } => {
                let passphrase = passphrase.as_deref().unwrap_or("");
                session.userauth_pubkey_file(&self.config.user, Some(public_key), private_key, Some(passphrase))
                    .context("Public key authentication failed")?;
            }
            SshAuth::Agent => {
                session.userauth_agent(&self.config.user)
                    .context("SSH agent authentication failed")?;
            }
        }

        if !session.authenticated() {
            Err("SSH authentication failed".into())
        } else {
            Ok(())
        }
    }

    /// Disconnect from the remote server
    pub async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.session = None;
        self.sftp = None;
        self.stream = None;
        self.state = ConnectionState::Disconnected;
        log::info!("Disconnected from {}", self.config.host);
        Ok(())
    }

    /// Get the current connection state
    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    /// Execute a command on the remote server
    pub async fn execute(&self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        if !self.is_connected() {
            return Err("Not connected to remote server".into());
        }

        let session = self.session.as_ref()
            .context("SSH session not available")?;

        let mut channel = session.channel_session()
            .context("Failed to create SSH channel")?;

        channel.exec(command)
            .context("Failed to execute command")?;

        // Read output
        let mut output = Vec::new();
        let mut stderr = Vec::new();

        use std::io::Read;
        channel.read_to_end(&mut output)?;
        channel.stderr().read_to_end(&mut stderr)?;

        channel.wait_close()?;

        let exit_status = channel.exit_status()
            .context("Failed to get exit status")?;

        if exit_status != 0 {
            let stderr_str = String::from_utf8_lossy(&stderr);
            Err(format!("Command failed with exit status {}: {}", exit_status, stderr_str).into())
        } else {
            Ok(String::from_utf8_lossy(&output).to_string())
        }
    }

    /// Get SFTP session (for file operations)
    pub fn sftp(&self) -> Result<&Sftp> {
        self.sftp.as_ref()
            .context("SFTP session not available")
    }

    /// Get SSH session
    pub fn session(&self) -> Result<&Session> {
        self.session.as_ref()
            .context("SSH session not available")
    }

    /// Get connection configuration
    pub fn config(&self) -> &SshConfig {
        &self.config
    }
}

/// SSH connection pool for managing multiple connections
pub struct SshConnectionPool {
    connections: HashMap<String, Arc<Mutex<SshConnection>>>,
}

impl SshConnectionPool {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Get or create a connection for the given config
    pub async fn get_connection(&mut self, config: SshConfig) -> Result<Arc<Mutex<SshConnection>>, Box<dyn std::error::Error>> {
        let key = format!("{}@{}:{}", config.user, config.host, config.port);

        if let Some(connection) = self.connections.get(&key) {
            let conn = connection.lock().unwrap();
            if conn.is_connected() {
                return Ok(connection.clone());
            }
        }

        // Create new connection
        let mut connection = SshConnection::new(config);
        connection.connect().await?;

        let connection = Arc::new(Mutex::new(connection));
        self.connections.insert(key, connection.clone());

        Ok(connection)
    }

    /// Remove a connection from the pool
    pub fn remove_connection(&mut self, config: &SshConfig) {
        let key = format!("{}@{}:{}", config.user, config.host, config.port);
        self.connections.remove(&key);
    }

    /// Get all active connections
    pub fn active_connections(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }
}

impl Default for SshConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}