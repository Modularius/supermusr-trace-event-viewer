use std::path::PathBuf;

use clap::Parser;


#[derive(Clone, Parser)]
pub(crate) struct OutputToFile {
    #[clap(long)]
    pub(crate) path: PathBuf,
}

impl OutputToFile {
    pub(crate) fn write_image() {
        
    }
}