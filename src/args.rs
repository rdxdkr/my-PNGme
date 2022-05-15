use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct PngMeArgs {
    #[clap(subcommand)]
    pub command_type: CommandType,
}

#[derive(Debug, Subcommand)]
pub enum CommandType {
    /// Encode a message in a PNG chunk
    Encode(EncodeArgs),
}

#[derive(Debug, Args)]
pub struct EncodeArgs {}
