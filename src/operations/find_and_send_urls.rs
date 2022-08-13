use hydrus_api::{wrapper::hydrus_file::HydrusFile, Hydrus};
use rustnao::Handler;
use tempdir::TempDir;

use crate::error::Result;
use crate::utils::pixiv::{get_sauces_for_file, get_urls};

pub async fn find_and_send_urls(
    hydrus: &Hydrus,
    handler: &Handler,
    tmpdir: &TempDir,
    file: &mut HydrusFile,
) -> Result<()> {
    let sauces = get_sauces_for_file(&handler, tmpdir, file).await?;
    let urls = get_urls(&sauces);
    for url in urls {
        hydrus.import().url(url).run().await?;
    }

    Ok(())
}
