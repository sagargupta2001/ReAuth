//! Defines fixed, internal constants for the SDK and plugin handshake.

/// The address the plugin's gRPC server will bind to.
/// Using port `0` tells the OS to assign a random, available port.
pub const PLUGIN_SERVER_BIND_ADDR: &str = "127.0.0.1:0";

/// The protocol version of the ReAuth Core.
pub const HANDSHAKE_CORE_VERSION: &str = "1";

/// The protocol version of the plugin system itself.
pub const HANDSHAKE_PROTOCOL_VERSION: &str = "1";

/// The network protocol used for communication (e.g., "tcp").
pub const HANDSHAKE_PROTOCOL_NETWORK: &str = "tcp";

/// The application protocol used for communication (e.g., "grpc").
pub const HANDSHAKE_PROTOCOL_TYPE: &str = "grpc";