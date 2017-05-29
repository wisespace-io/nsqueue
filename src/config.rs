#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    // Identifiers sent to nsqd representing this client
    client_id: Option<String>,
    short_id: Option<String>,
    long_id: Option<String>,
    hostname: Option<String>,
    user_agent: String,

    // Compression Settings
    deflate: bool,
    deflate_level: u16,
    snappy: bool,

    feature_negotiation: bool,

    // Duration of time between heartbeats.
    heartbeat_interval: u32,

    // Timeout used by nsqd before flushing buffered writes (set to 0 to disable).
    message_timeout: u32,

    // Size of the buffer (in bytes) used by nsqd for buffering writes to this connection
    output_buffer_size: u64,
    output_buffer_timeout: u32,

    // Integer percentage to sample the channel (requires nsqd 0.2.25+)
    sample_rate: u16,

    // tls_v1 - Bool enable TLS negotiation
    tls_v1: bool,
}
use hostname::get_hostname;

#[allow(dead_code)]
impl Config {
    pub fn default() -> Config {
        Config {
            client_id: get_hostname(),
            short_id: get_hostname(),
            long_id: get_hostname(),
            user_agent: String::from("github.com/wisespace-io/nsqueue"),
            hostname: get_hostname(),
            deflate: false,
            deflate_level: 6,
            snappy: false,
            feature_negotiation: true,
            heartbeat_interval: 30000,
            message_timeout: 0,
            output_buffer_size: 16384,
            output_buffer_timeout: 250,
            sample_rate: 0,
            tls_v1: false,
        }
    }

    pub fn client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn hostname(mut self, hostname: String) -> Self {
        self.hostname = Some(hostname);
        self
    }

    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn snappy(mut self, snappy: bool) -> Self {
        self.snappy = snappy;
        self
    }    
}