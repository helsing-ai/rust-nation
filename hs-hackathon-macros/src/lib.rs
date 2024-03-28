use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Attribute macro for your solutions `async fn main` to setup loggig, tokio, verifies the
/// environment etc
#[proc_macro_attribute]
pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
    let main = parse_macro_input!(input as ItemFn);

    let sig = main.sig;
    let block = main.block;

    quote! {
        #[::hs_hackathon::prelude::tokio::main]
        #sig {
            {
                use hs_hackathon::prelude::{tracing, tracing_subscriber};
                use tracing::level_filters::LevelFilter;
                use tracing_subscriber::EnvFilter;

                let filter = EnvFilter::builder()
                    .with_default_directive(LevelFilter::DEBUG.into())
                    .from_env().expect("internal error: failed to setup tracing");

                tracing_subscriber::fmt().with_env_filter(filter).init();

                #[cfg(target_os = "linux")]
                if cfg!(debug_assertions) {
                    panic!("running in debug mode is not supported on the pi due to performance restrictions");
                }

                #[cfg(target_os = "linux")]
                {
                    const DRONE_SERVICE: &str = "drone-wifi";

                    let status = std::process::Command::new("systemctl")
                        .arg("is-active")
                        .arg(DRONE_SERVICE)
                        .status()
                        .expect("failed to verify the wifi of the drones wifi");

                    assert!(status.success(), "drone wifi systemd service has a unexpected status");
                    tracing::info!("network setup: success");
                }
            }

            #block
        }
    }
    .into()
}
