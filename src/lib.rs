pub mod cli;
pub mod config;
pub mod req;
pub mod utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraArgs {
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
}
