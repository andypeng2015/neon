use std::{ffi::CString, path::Path, str::FromStr, sync::Arc};

use rand::Rng;
use safekeeper::simlib::{
    network::{Delay, NetworkOptions},
    proto::AnyMessage,
    world::World,
    world::{Node, NodeEvent},
};
use tracing::{info, warn};
use utils::{id::TenantTimelineId, lsn::Lsn};

use crate::{
    bindings::{
        neon_tenant_walproposer, neon_timeline_walproposer, sim_redo_start_lsn, syncSafekeepers,
        wal_acceptor_connection_timeout, wal_acceptor_reconnect_timeout, wal_acceptors_list,
        MyInsertRecord, WalProposerCleanup, WalProposerRust,
    },
    c_context,
    simtest::{
        log::{init_logger, SimClock},
        safekeeper::run_server,
        util::{generate_schedule, TestConfig},
    },
};

use super::{
    disk::Disk,
    util::{Schedule, TestAction},
};

#[test]
fn sync_empty_safekeepers() {
    let clock = init_logger();
    let mut config = TestConfig::new(Some(clock));
    let test = config.start(1337);

    let lsn = test.sync_safekeepers().unwrap();
    assert_eq!(lsn, Lsn(0));
    info!("Sucessfully synced empty safekeepers at 0/0");

    let lsn = test.sync_safekeepers().unwrap();
    assert_eq!(lsn, Lsn(0));
    info!("Sucessfully synced (again) empty safekeepers at 0/0");
}

#[test]
fn run_walproposer_generate_wal() {
    let clock = init_logger();
    let mut config = TestConfig::new(Some(clock));
    // config.network.timeout = Some(250);
    let test = config.start(1337);

    let lsn = test.sync_safekeepers().unwrap();
    assert_eq!(lsn, Lsn(0));
    info!("Sucessfully synced empty safekeepers at 0/0");

    let mut wp = test.launch_walproposer(lsn);

    test.poll_for_duration(30);

    for i in 0..100 {
        wp.write_tx();
        test.poll_for_duration(5);
        wp.update();
    }
}

#[test]
fn crash_safekeeper() {
    let clock = init_logger();
    let mut config = TestConfig::new(Some(clock));
    // config.network.timeout = Some(250);
    let test = config.start(1337);

    let lsn = test.sync_safekeepers().unwrap();
    assert_eq!(lsn, Lsn(0));
    info!("Sucessfully synced empty safekeepers at 0/0");

    let mut wp = test.launch_walproposer(lsn);

    test.poll_for_duration(30);
    wp.update();

    wp.write_tx();
    wp.write_tx();
    wp.write_tx();

    test.servers[0].restart();

    test.poll_for_duration(100);
    wp.update();

    test.poll_for_duration(1000);
    wp.update();
}

#[test]
fn test_simple_restart() {
    let clock = init_logger();
    let mut config = TestConfig::new(Some(clock));
    // config.network.timeout = Some(250);
    let test = config.start(1337);

    let lsn = test.sync_safekeepers().unwrap();
    assert_eq!(lsn, Lsn(0));
    info!("Sucessfully synced empty safekeepers at 0/0");

    let mut wp = test.launch_walproposer(lsn);

    test.poll_for_duration(30);
    wp.update();

    wp.write_tx();
    wp.write_tx();
    wp.write_tx();
    test.poll_for_duration(100);
    wp.update();

    wp.stop();
    drop(wp);

    let lsn = test.sync_safekeepers().unwrap();
    info!("Sucessfully synced safekeepers at {}", lsn);
}

#[test]
fn test_simple_schedule() -> anyhow::Result<()> {
    let clock = init_logger();
    let mut config = TestConfig::new(Some(clock));
    config.network.keepalive_timeout = Some(100);
    let test = config.start(1337);

    let schedule: Schedule = vec![
        (0, TestAction::RestartWalProposer),
        (50, TestAction::WriteTx(5)),
        (100, TestAction::RestartSafekeeper(0)),
        (100, TestAction::WriteTx(5)),
        (110, TestAction::RestartSafekeeper(1)),
        (110, TestAction::WriteTx(5)),
        (120, TestAction::RestartSafekeeper(2)),
        (120, TestAction::WriteTx(5)),
        (201, TestAction::RestartWalProposer),
        (251, TestAction::RestartSafekeeper(0)),
        (251, TestAction::RestartSafekeeper(1)),
        (251, TestAction::RestartSafekeeper(2)),
        (251, TestAction::WriteTx(5)),
        (255, TestAction::WriteTx(5)),
        (1000, TestAction::WriteTx(5)),
    ];

    test.run_schedule(&schedule)?;
    info!("Test finished, stopping all threads");
    test.world.stop_all();

    Ok(())
}

#[test]
fn test_random_schedules() -> anyhow::Result<()> {
    let clock = init_logger();
    let mut config = TestConfig::new(Some(clock));
    config.network.keepalive_timeout = Some(100);

    for i in 0..1000 {
        let seed: u64 = rand::thread_rng().gen();
        let test = config.start(seed);
        warn!("Running test with seed {}", seed);

        let schedule = generate_schedule(seed);
        test.run_schedule(&schedule)?;

        test.world.stop_all();
    }

    Ok(())
}

#[test]
fn test_one_schedule() -> anyhow::Result<()> {
    let clock = init_logger();
    let mut config = TestConfig::new(Some(clock));
    config.network.keepalive_timeout = Some(100);

    // let seed = 6762900106769428342;
    // let test = config.start(seed);
    // warn!("Running test with seed {}", seed);

    // let schedule = generate_schedule(seed);
    // info!("schedule: {:?}", schedule);
    // test.run_schedule(&schedule)?;
    // test.world.stop_all();

    let seed = 14035854184686918762;
    let test = config.start(seed);
    warn!("Running test with seed {}", seed);

    let schedule = generate_schedule(seed);
    info!("schedule: {:?}", schedule);
    test.run_schedule(&schedule).unwrap();
    test.world.stop_all();

    Ok(())
}
