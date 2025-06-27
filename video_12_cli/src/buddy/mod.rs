mod config;

use std::{
    fs,
    path::{Path, PathBuf},
};

use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    ais::{
        OaClient,
        asst::{self, AsstId, ThreadId, VectorStoresId},
        new_oa_client,
    },
    buddy::config::Config,
    utils::files::{
        bundle_to_file, ensure_dir, list_files, load_from_json, load_from_toml, read_to_string,
        save_to_json,
    },
};

const BUDDY_TOML: &str = "buddy.toml";

#[derive(Debug)]
pub struct Buddy {
    dir: PathBuf,
    oac: OaClient,
    asst_id: AsstId,
    vs_id: VectorStoresId,
    config: Config,
}

#[derive(Debug, From, Deref, Deserialize, Serialize)]
pub struct Conv {
    thread_id: ThreadId,
}

impl Buddy {
    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub async fn init_from_dir(
        dir: impl AsRef<Path>,
        recreate_asst: bool,
        recreate_vs: bool,
    ) -> Result<Self> {
        let dir = dir.as_ref();

        let config: Config = load_from_toml(dir.join(BUDDY_TOML))?;

        let oac = new_oa_client()?;
        let vs_id = asst::load_or_create_vs(&oac, (&config).into(), recreate_vs).await?;
        let asst_id =
            asst::load_or_create_asst(&oac, (&config).into(), &vs_id, recreate_asst).await?;
        let buddy = Buddy {
            dir: dir.to_path_buf(),
            oac,
            vs_id,
            asst_id,
            config,
        };
        buddy.upload_instructions().await?;
        buddy.upload_files(false).await?;

        Ok(buddy)
    }

    pub async fn upload_instructions(&self) -> Result<bool> {
        let file = self.dir.join(&self.config.instructions_file);
        if file.exists() {
            let inst_content = read_to_string(&file)?;
            asst::upload_instructions(&self.oac, &self.asst_id, inst_content).await?;
            println!("Instructions uploaded");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn chat(&self, conv: &Conv, msg: &str) -> Result<String> {
        let res = asst::run_thread_msg(&self.oac, &self.asst_id, &conv.thread_id, msg).await?;

        Ok(res)
    }

    pub async fn upload_files(&self, recreate: bool) -> Result<u32> {
        let mut num_uploaded = 0;
        let data_files_dir = self.data_files_dir()?;
        let exclude_element = format!("*{}*", &self.asst_id);

        for file in list_files(
            &data_files_dir,
            Some(&["*.rs", "*.md"]),
            Some(&[&exclude_element]),
        )? {
            let file_str = file.to_string_lossy();
            if !file_str.contains(".buddy") {
                return Err(format!("Error should not delete: {file_str}",).into());
            }
            fs::remove_file(&file)?;
        }

        for bundle in self.config.file_bundles.iter() {
            let src_dir = self.dir.join(&bundle.src_dir);

            if src_dir.is_dir() {
                let src_globs: Vec<&str> = bundle.src_globs.iter().map(AsRef::as_ref).collect();

                let files = list_files(&src_dir, Some(&src_globs), None)?;

                if !files.is_empty() {
                    // TO FIX
                    let ext = if bundle.dst_ext == "rs" {
                        &"txt".to_string()
                    } else {
                        &bundle.dst_ext
                    };

                    let bundle_file_name = format!(
                        "{}-{}-bundle-{}.{}",
                        self.name(),
                        bundle.bundle_name,
                        self.asst_id,
                        ext
                    );

                    let bundle_file = self.data_files_dir()?.join(bundle_file_name);
                    let force_reupload = recreate || !bundle_file.exists();
                    bundle_to_file(files, &bundle_file)?;

                    let (_, uploaded) = asst::upload_file_by_name(
                        &self.oac,
                        &self.vs_id,
                        &bundle_file,
                        force_reupload,
                    )
                    .await?;

                    if uploaded {
                        num_uploaded += 1;
                    }
                }
            }
        }

        Ok(num_uploaded)
    }

    pub async fn load_or_create_conv(&self, recreate: bool) -> Result<Conv> {
        let conv_file = self.data_dir()?.join("conv.json");

        if recreate && conv_file.exists() {
            fs::remove_file(&conv_file)?;
        }

        let conv = if let Ok(conv) = load_from_json::<Conv>(&conv_file) {
            asst::get_thread(&self.oac, &conv.thread_id)
                .await
                .map_err(|_| format!("Cannot find thread_id for {conv:?} "))?;
            println!("Conversation loaded");
            conv
        } else {
            let thread_id = asst::create_thread(&self.oac).await?;
            println!("Conversation created");
            let conv = thread_id.into();
            save_to_json(&conv_file, &conv)?;
            conv
        };

        Ok(conv)
    }
}

impl Buddy {
    fn data_dir(&self) -> Result<PathBuf> {
        let data_dir = self.dir.join(".buddy");
        ensure_dir(&data_dir)?;
        Ok(data_dir)
    }

    fn data_files_dir(&self) -> Result<PathBuf> {
        let dir = self.data_dir()?.join("files");
        ensure_dir(&dir)?;
        Ok(dir)
    }
}
