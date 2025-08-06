//! Wasmtime runtime
//! No more in use (Replased by WasmEdge runtime)

use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::p2::WasiCtxBuilder;
use wasmtime_wasi::preview1::{self, WasiP1Ctx};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wasm_dir = std::env::current_dir()?;
    wasm_dir.push("target/wasm32-wasip1/release/corelib.wasm");

    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config)?;

    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
    preview1::add_to_linker_async(&mut linker, |t| t)?;

    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .allow_tcp(true)
        .allow_udp(true)
        .inherit_network()
        .build_p1();

    let mut store = Store::new(&engine, wasi_ctx);

    let module = Module::from_file(&engine, wasm_dir)?;
    let instance = linker.instantiate_async(&mut store, &module).await?;

    let func = instance.get_typed_func::<(), ()>(&mut store, "run")?;

    func.call_async(&mut store, ()).await?;

    Ok(())
}
