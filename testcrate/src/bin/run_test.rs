use std::{fs, path::PathBuf};

use clap::Parser;
use common::{
    command::{assert_dir_exists, ComplexCommand},
    container_network::{Container, ContainerNetwork},
    test_mode::TestMode,
};

/// `docker-compose` is not programmable among other problems. What this does is
/// create an `--internal` network of containers using locally built binaries.
/// Logs from `stdout` and `stderr` are actively pushed to log files under
/// `testcrate/logs`. Most failure cases are all caught and all containers are
/// force stopped on failure or finish.
///
/// The overall run waits on the last container in the list finishing
#[derive(Parser)]
#[clap(version)]
pub struct CliArgs {
    #[clap(long, default_value = "./")]
    /// The base directory that has `onomy-rs/testcrate/logs`
    pub dir: String,
    #[clap(long, default_value = "testnet")]
    /// The name of the network used
    pub network: String,
    #[clap(long, default_value = "x86_64-unknown-linux-gnu")]
    /// Target used for binaries that will run in the containers
    pub target: String,
    #[clap(long)]
    /// Turns on "CI mode" which will redirects all output that would go to log
    /// files to stdout and stderr instead
    pub ci: bool,
    #[clap(arg_enum)]
    pub test_mode: TestMode,
}

// TODO ctrlc shutdown

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    let ci = args.ci;
    // check the directory for expected folders we will be using
    let base_dir = PathBuf::from(&args.dir);
    assert_dir_exists(&base_dir).unwrap();
    let base_dir = fs::canonicalize(base_dir).unwrap();
    let base_dir_s = base_dir.to_str().unwrap();
    println!("using base directory {}", base_dir_s);
    assert!(base_dir_s.ends_with("onomy-rs"));

    let log_dir = base_dir.join("testcrate/logs");
    assert_dir_exists(&log_dir).unwrap();

    println!("running `cargo build --release --target {}`", args.target);
    ComplexCommand::new(
        "cargo",
        &["build", "--release", "--target", &args.target],
        ci,
    )
    .unwrap()
    .stderr_to_file(&log_dir.join("cmd_cargo_build_err.log"))
    .await
    .unwrap()
    .wait()
    .await
    .unwrap();

    // after the build we should have a release directory with the binaries
    let bin_dir = base_dir.join(format!("target/{}/release", args.target));
    assert_dir_exists(&bin_dir).unwrap();

    let base_image = "fedora:36";

    let mut cn = ContainerNetwork {
        network_name: args.network,
        containers: vec![],
        log_dir,
    };
    match args.test_mode {
        mode @ (TestMode::Health | TestMode::GetResponse) => {
            cn.containers.push(Container::new(
                "equity_core",
                base_image,
                &bin_dir.join("equity_core"),
                "--listener=0.0.0.0:4040",
            ));
            cn.containers.push(Container::new(
                "test_runner",
                base_image,
                &bin_dir.join("test_runner"),
                mode.typed(),
            ));
        }
    }
    cn.run(args.ci).await.unwrap();
}
