use tokio::task::JoinSet;
use video_2_ai_fc::Error;
use video_2_ai_fc::{conv, oa_client::new_oa_client, tools::new_ai_tools};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let oa_client = new_oa_client()?;
    let ai_tools = new_ai_tools()?;

    let questions = &[
        "Convert 3 Dollars to Euro",
        "why is the sky red (be concise)",
        "Convert 5 Euro to Hryvnia",
    ];

    let mut join_set: JoinSet<(String, Result<String, Error>)> = JoinSet::new();

    for &q in questions {
        let oa_client = oa_client.clone();
        let ai_tools = ai_tools.clone();

        join_set.spawn(async move {
            let response = conv::send_user_msg(oa_client.clone(), ai_tools.clone(), q).await;
            (q.to_string(), response)
        });
    }

    while let Some(join_res) = join_set.join_next().await {
        let (question, result) = join_res?;
        let result = result?;

        println!("Question: {question}\nResponse: {result}\n\n");
    }

    Ok(())
}
