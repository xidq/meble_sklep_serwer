use std::sync::OnceLock;

pub static PEPPER_KEY: OnceLock<String> = OnceLock::new();
/// Path for folder with external files eg images.
pub static FILES_LOCATION: OnceLock<String> = OnceLock::new();
pub static FRONT_SERV_ADDRESS: OnceLock<String> = OnceLock::new();
pub static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();
pub static DATABASE_URL: OnceLock<String> = OnceLock::new();
pub static TLS_CERT_PATH: OnceLock<String> = OnceLock::new();
pub static TLS_KEY_PATH: OnceLock<String> = OnceLock::new();
pub static CURRENT_PORT: OnceLock<String> = OnceLock::new();
pub static CURRENT_ADDRESS: OnceLock<String> = OnceLock::new();
pub static GOVERNOR_RATE_LIMIT: OnceLock<u64> = OnceLock::new();
pub static GOVERNOR_BURST_SIZE: OnceLock<u32> = OnceLock::new();

pub trait OnceLockExt<T> {
    fn v(&self, monit: &str) -> &T;
}

impl<T> OnceLockExt<T> for OnceLock<T> {
    fn v(&self, monit: &str) -> &T {
        let panic = if monit.is_empty() {
            "OnceLock accessed before initialization"
        } else {
            monit
        };
        self.get().expect(panic)
    }

}