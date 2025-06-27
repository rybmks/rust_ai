use crate::{
    Result,
    ais::{
        OaClient,
        msg::{get_text_content, user_msg},
    },
    utils::files::XFile,
};
use async_openai::types::{
    AssistantObject, AssistantTools, AssistantToolsFileSearch, CreateAssistantRequest,
    CreateAssistantToolFileSearchResources, CreateAssistantToolResources, CreateRunRequest,
    CreateThreadRequest, CreateVectorStoreFileRequest, ModifyAssistantRequest, RunStatus,
    ThreadObject,
};
use async_openai::types::{CreateFileRequest, FilePurpose};
use console::Term;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::Duration,
};
const DEFAULT_QUERY: &[(&str, &str)] = &[("limit", "100")];
const FILES_QUERY: &[(&str, &str)] = &[("purpose", "assistants")];
const POLLING_DURATION_MS: u64 = 500;

#[derive(Debug, From, Deref, Display)]
pub struct AsstId(String);

#[derive(Debug, From, Deref, Display, Serialize, Deserialize)]
pub struct ThreadId(String);

#[derive(Debug, From, Deref, Display)]
pub struct FileId(String);

#[derive(Debug, From, Deref, Display)]
pub struct VectorStoresId(String);

pub struct CreateConfig {
    pub name: String,
    pub model: String,
}

pub async fn create(
    oac: &OaClient,
    config: CreateConfig,
    vs_id: &VectorStoresId,
) -> Result<AsstId> {
    let oa_assts = oac.assistants();

    let create_request = CreateAssistantRequest {
        model: config.model.clone(),
        name: Some(config.name.clone()),
        tools: Some(vec![AssistantTools::FileSearch(
            AssistantToolsFileSearch::default(),
        )]),

        tool_resources: Some(CreateAssistantToolResources {
            file_search: Some(CreateAssistantToolFileSearchResources {
                vector_store_ids: Some(vec![vs_id.to_string()]),
                ..Default::default()
            }),
            code_interpreter: None,
        }),

        ..Default::default()
    };

    let asst_obj = oa_assts.create(create_request).await?;
    Ok(asst_obj.id.into())
}

pub async fn load_or_create_asst(
    oac: &OaClient,
    config: CreateConfig,
    vs_id: &VectorStoresId,
    recreate: bool,
) -> Result<AsstId> {
    let asst_obj = first_by_name(oac, &config.name).await?;
    let mut asst_id = asst_obj.map(|o| AsstId::from(o.id));

    if let (true, Some(asst_id_ref)) = (recreate, asst_id.as_ref()) {
        delete(oac, asst_id_ref).await?;
        asst_id.take();
        tracing::info!("Assistant {} deleted", config.name);
    }

    if let Some(asst_id) = asst_id {
        tracing::info!("Assistant {} loaded", config.name);
        Ok(asst_id)
    } else {
        let name = config.name.clone();
        let asst_id = create(oac, config, vs_id).await?;
        tracing::info!("Assistant {} created", name);
        Ok(asst_id)
    }
}

pub async fn load_or_create_vs(
    oac: &OaClient,
    config: CreateConfig,
    recreate: bool,
) -> Result<VectorStoresId> {
    let oa_vs = oac.vector_stores();

    let vs_obj = {
        let vss = oa_vs.list(DEFAULT_QUERY).await?.data;
        vss.into_iter()
            .find(|vs| vs.name.as_ref().map(|n| n == &config.name).unwrap_or(false))
    };
    let mut vs_id = vs_obj.map(|o| VectorStoresId::from(o.id));

    if let (true, Some(vs_id_ref)) = (recreate, vs_id.as_ref()) {
        oa_vs.delete(vs_id_ref).await?;
        vs_id.take();
        tracing::info!("Vector store {} deleted", config.name);
    }

    if let Some(vs_id) = vs_id {
        tracing::info!("Vector store {} loaded", config.name);
        Ok(vs_id)
    } else {
        let name = config.name.clone();
        let vs_obj = oa_vs
            .create(async_openai::types::CreateVectorStoreRequest {
                name: Some(config.name),
                ..Default::default()
            })
            .await?;
        tracing::info!("Vector store {} created", name);
        Ok(VectorStoresId::from(vs_obj.id))
    }
}

pub async fn first_by_name(oac: &OaClient, name: &str) -> Result<Option<AssistantObject>> {
    let oa_assts = oac.assistants();
    let assts = oa_assts.list(DEFAULT_QUERY).await?.data;
    let asst_obj = assts
        .into_iter()
        .find(|a| a.name.as_ref().map(|n| n == name).unwrap_or(false));
    Ok(asst_obj)
}

pub async fn delete(oac: &OaClient, asst_id: &AsstId) -> Result<()> {
    let oa_assts = oac.assistants();

    oa_assts.delete(asst_id).await?;

    Ok(())
}

pub async fn upload_instructions(
    oac: &OaClient,
    asst_id: &AsstId,
    inst_content: String,
) -> Result<()> {
    let oa_assts = oac.assistants();
    let modif = ModifyAssistantRequest {
        instructions: Some(inst_content),
        ..Default::default()
    };
    oa_assts.update(asst_id, modif).await?;

    Ok(())
}

pub async fn create_thread(oac: &OaClient) -> Result<ThreadId> {
    let oa_threads = oac.threads();
    let res = oa_threads.create(CreateThreadRequest::default()).await?;
    Ok(res.id.into())
}

pub async fn get_thread(oac: &OaClient, thread_id: &ThreadId) -> Result<ThreadObject> {
    let oa_threads = oac.threads();
    let thread_obj = oa_threads.retrieve(thread_id).await?;

    Ok(thread_obj)
}

pub async fn run_thread_msg(
    oac: &OaClient,
    asst_id: &AsstId,
    thread_id: &ThreadId,
    msg: &str,
) -> Result<String> {
    let msg = user_msg(msg);

    let _message_obj = oac.threads().messages(thread_id).create(msg).await?;
    let run_request = CreateRunRequest {
        assistant_id: asst_id.to_string(),
        ..Default::default()
    };
    let run = oac.threads().runs(thread_id).create(run_request).await?;
    let term = Term::stdout();
    loop {
        print!(">");
        let run = oac.threads().runs(thread_id).retrieve(&run.id).await?;
        print!("<");
        match run.status {
            RunStatus::Completed => {
                term.write_line("")?;
                return get_first_thread_msg_content(oac, thread_id).await;
            }
            RunStatus::Queued | RunStatus::InProgress => (),
            other => {
                term.write_line("")?;
                return Err(format!("Error while run: {other:?}").into());
            }
        }
        tokio::time::sleep(Duration::from_millis(POLLING_DURATION_MS)).await;
    }
}

pub async fn get_first_thread_msg_content(oac: &OaClient, thread_id: &ThreadId) -> Result<String> {
    static QUERY: [(&str, &str); 1] = [("limit", "1")];
    let messages = oac.threads().messages(thread_id).list(&QUERY).await?;
    let msg = messages
        .data
        .into_iter()
        .next()
        .ok_or("No message found".to_string())?;
    let text = get_text_content(msg)?;
    Ok(text)
}

pub async fn get_files_hashmap(
    oac: &OaClient,
    vs_id: &VectorStoresId,
) -> Result<HashMap<String, FileId>> {
    let oas_vs = oac.vector_stores();
    let oa_vs_files = oas_vs.files(vs_id);
    let asst_files = oa_vs_files.list(DEFAULT_QUERY).await?.data;
    let asst_file_ids: HashSet<String> = asst_files.into_iter().map(|f| f.id).collect();

    let oa_files = oac.files();
    let org_files = oa_files.list(FILES_QUERY).await?.data;

    let file_id_by_name: HashMap<String, FileId> = org_files
        .into_iter()
        .filter(|org_file| asst_file_ids.contains(&org_file.id))
        .map(|org_file| (org_file.filename, org_file.id.into()))
        .collect();

    Ok(file_id_by_name)
}

pub async fn upload_file_by_name(
    oac: &OaClient,
    asst_id: &VectorStoresId,
    file: &Path,
    force: bool,
) -> Result<(FileId, bool)> {
    let file_name = file.x_file_name();
    let mut file_id_by_name = get_files_hashmap(oac, asst_id).await?;

    let file_id = file_id_by_name.remove(file_name);
    if !force && let Some(file_id) = file_id {
        return Ok((file_id, false));
    }

    if let Some(file_id) = file_id {
        let oa_files = oac.files();
        if let Err(err) = oa_files.delete(&file_id).await {
            println!("Can't delete file: {err}\n");
        }

        let oa_vs = oac.vector_stores();
        let oa_vs_files = oa_vs.files(asst_id);
        if let Err(err) = oa_vs_files.delete(&file_id).await {
            println!("Cant delete assistant file: {err}\n");
        }
    }

    let term = Term::stdout();

    term.write_line(&format!("Uploading file {}", file.x_file_name()))?;
    let oa_files = oac.files();
    let oa_file = oa_files
        .create(CreateFileRequest {
            file: file.into(),
            purpose: FilePurpose::Assistants,
        })
        .await?;

    term.clear_last_lines(1)?;
    term.write_line(&format!("Uploaded file {}", file.x_file_name()))?;

    let oa_vs = oac.vector_stores();
    let oa_vs_files = oa_vs.files(asst_id);
    let asst_file_obj = oa_vs_files
        .create(CreateVectorStoreFileRequest {
            file_id: oa_file.id.clone(),
            ..Default::default()
        })
        .await?;

    if oa_file.id != asst_file_obj.id {
        println!("File id not matching {} {}", oa_file.id, asst_file_obj.id)
    }

    Ok((asst_file_obj.id.into(), true))
}
