use rust_bert::pipelines::common::ModelResource;
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::text_generation::TextGenerationModel;
use rust_bert::resources::ResourceProvider;
use rust_bert::{
    gpt_neo::{
        GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
    },
    pipelines::text_generation::TextGenerationConfig,
    resources::RemoteResource,
};
fn main() {
    let model_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoModelResources::GPT_NEO_2_7B,
    ));
    let model_resource = ModelResource::Torch(model_resource);

    let config_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoConfigResources::GPT_NEO_2_7B,
    ));
    let vocab_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoVocabResources::GPT_NEO_2_7B,
    ));
    let merges_resource: Option<Box<dyn ResourceProvider + Send>> = Some(Box::new(
        RemoteResource::from_pretrained(GptNeoMergesResources::GPT_NEO_2_7B),
    ));

    let generation_config = TextGenerationConfig {
        model_type: ModelType::GPTNeo,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource,
        num_beams: 5,
        no_repeat_ngram_size: 2,
        max_length: Some(100),
        ..Default::default()
    };

    let model = TextGenerationModel::new(generation_config).unwrap();

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let split = line.split('/').collect::<Vec<&str>>();
        let slc = split.as_slice();
        let output = model.generate(&slc[1..], Some(slc[0])).unwrap();
        for sentence in output {
            println!("{sentence}"
        );
        }
    }
}
