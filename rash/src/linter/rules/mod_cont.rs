// Determinism rules (bashrs-specific)
pub mod det001;
pub mod det002;
pub mod det003;
pub mod det004;

// Idempotency rules (bashrs-specific)
pub mod idem001;
pub mod idem002;
pub mod idem003;

// Best practice rules (bashrs-specific)
pub mod bash001;
pub mod bash002;
pub mod bash003;
pub mod bash004;
pub mod bash005;
pub mod bash006;
pub mod bash007;
pub mod bash008;
pub mod bash009;
pub mod bash010;

// Security rules (bashrs-specific)
pub mod sec001;
pub mod sec002;
pub mod sec003;
pub mod sec004;
pub mod sec005;
pub mod sec006;
pub mod sec007;
pub mod sec008;
pub mod sec009;
pub mod sec010;
pub mod sec010_logic;
pub mod sec011;
pub mod sec012;
pub mod sec013;
pub mod sec014;
pub mod sec015;
pub mod sec016;
pub mod sec017;
pub mod sec018;
pub mod sec019;
pub mod sec020;
pub mod sec021;
pub mod sec022;
pub mod sec023;
pub mod sec024;

// Makefile-specific rules (bashrs-specific)
pub mod make001;
pub mod make002;
pub mod make003;
pub mod make004;
pub mod make005;
pub mod make006;
pub mod make007;
pub mod make008;
pub mod make009;
pub mod make010;
pub mod make011;
pub mod make012;
pub mod make013;
pub mod make014;
pub mod make015;
pub mod make016;
pub mod make017;
pub mod make018;
pub mod make019;
pub mod make020;

// Dockerfile rules
pub mod docker001;
pub mod docker002;
pub mod docker003;
pub mod docker004;
pub mod docker005;
pub mod docker006;
pub mod docker007; // F061: Shell entrypoint detection
pub mod docker008; // F062, F064, F072: Shell in CMD/RUN detection
pub mod docker009; // F063: Multi-stage build validation
pub mod docker010; // F065: HEALTHCHECK validation
pub mod docker010_logic;
pub mod docker011; // F069: USER directive validation
pub mod docker012; // F075: STOPSIGNAL validation

// Coursera Lab Image rules (profile-based)
pub mod coursera;

// macOS launchd plist validation rules (F076-F085)
pub mod launchd001;

// Signal and process management rules (F096-F100)
pub mod signal001;

// Dev Container validation rules
pub mod devcontainer;
pub mod devcontainer_logic;

// systemd unit file validation rules (F086-F095)
pub mod systemd001;

// Performance rules (PERF001-PERF005)
pub mod perf001;
pub mod perf002;
pub mod perf003;
pub mod perf004;
pub mod perf005;

// Portability rules (PORT001-PORT005)
pub mod port001;
pub mod port002;
pub mod port003;
pub mod port004;
pub mod port005;

// Reliability rules (REL001-REL005)
pub mod rel001;
pub mod rel002;
pub mod rel003;
pub mod rel004;
pub mod rel005;

use crate::linter::LintResult;

include!("mod_lint_2.rs");
