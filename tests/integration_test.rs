// Integration test for SigmaOS-scheduler
// Verifies the crate builds and links as a dependency.
#![allow(unused_imports)]

use sigmaos_scheduler as _;

#[test]
fn crate_links_and_runs() {
    // The crate compiled and linked; exercise a no-op assertion.
    assert_eq!(2 + 2, 4);
}
