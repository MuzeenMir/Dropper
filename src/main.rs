// Entry point for the `sentinel` CLI.
//
// Pre-v0.1 stub. Subcommands `install`, `doctor`, `tail` land alongside
// the v0.1 sprint work tracked in TODOS.md. `service` (this PR) starts
// the resolver + block-page server on the local machine.

use std::process::ExitCode;

use anyhow::Result;
use sentinel::blockpage::AppState;
use sentinel::feed::{new_blocklist, run_urlhaus_refresher};
use sentinel::resolver::{self, Resolver};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("sentinel {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }

    if args.first().map(|s| s.as_str()) == Some("service") {
        return match run_service() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("sentinel service: {e:#}");
                ExitCode::FAILURE
            }
        };
    }

    eprintln!("sentinel: pre-v0.1 skeleton.");
    eprintln!("subcommands `install`, `doctor`, `tail` land in the v0.1 sprint.");
    eprintln!("see TODOS.md and DESIGN.md for scope.");
    eprintln!();
    print_help();
    ExitCode::from(2)
}

fn print_help() {
    println!("sentinel — open-source DNS shield for Windows");
    println!();
    println!("usage: sentinel [--version | -V] [--help | -h]");
    println!("       sentinel service");
    println!();
    println!("commands:");
    println!("  service    run the local DNS resolver + block-page server");
    println!();
    println!("status: pre-v0.1, only `service` is wired. see TODOS.md.");
}

/// Boot a multi-threaded tokio runtime, then run the resolver and
/// block-page server in parallel until either exits.
fn run_service() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let blocklist = new_blocklist();
        let blockpage = AppState::new();

        let resolver = Resolver::new(blocklist.clone(), blockpage.clone());

        // All three tasks should run for the lifetime of the process. If
        // any returns we propagate so the supervisor (Windows service /
        // systemd / installer) knows to restart.
        let resolver_task = tokio::spawn(async move { resolver::serve(resolver).await });
        let blockpage_task =
            tokio::spawn(async move { sentinel::blockpage::serve(blockpage).await });
        let refresher_task = {
            let blocklist = blocklist.clone();
            tokio::spawn(async move { run_urlhaus_refresher(blocklist).await })
        };

        tokio::select! {
            r = resolver_task => match r {
                Ok(inner) => inner,
                Err(e) => Err(anyhow::anyhow!("resolver task panicked: {e}")),
            },
            r = blockpage_task => match r {
                Ok(inner) => inner,
                Err(e) => Err(anyhow::anyhow!("blockpage task panicked: {e}")),
            },
            r = refresher_task => match r {
                Ok(inner) => inner,
                Err(e) => Err(anyhow::anyhow!("urlhaus refresher task panicked: {e}")),
            },
        }
    })
}
