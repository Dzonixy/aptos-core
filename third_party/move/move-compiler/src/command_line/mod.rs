// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod compiler;

pub const DEPENDENCY: &str = "dependency";
pub const DEPENDENCY_SHORT: char = 'd';

pub const SENDER: &str = "sender";
pub const SENDER_SHORT: char = 's';

pub const OUT_DIR: &str = "out-dir";
pub const OUT_DIR_SHORT: char = 'o';
pub const DEFAULT_OUTPUT_DIR: &str = "build";

pub const SHADOW: &str = "shadow";
pub const SHADOW_SHORT: char = 'S';

pub const SKIP_ATTRIBUTE_CHECKS: &str = "skip-attribute-checks";

pub const SOURCE_MAP: &str = "source-map";
pub const SOURCE_MAP_SHORT: char = 'm';

pub const TEST: &str = "test";
pub const TEST_SHORT: char = 't';

pub const VERIFY: &str = "verify";
pub const VERIFY_SHORT: char = 'v';

pub const FLAVOR: &str = "flavor";

pub const BYTECODE_VERSION: &str = "bytecode-version";

pub const COLOR_MODE_ENV_VAR: &str = "COLOR_MODE";

pub const MOVE_COMPILED_INTERFACES_DIR: &str = "mv_interfaces";

pub const COMPILED_NAMED_ADDRESS_MAPPING: &str = "compiled-module-address-name";

pub const MOVE_COMPILER_DEBUG_ENV_VAR: &str = "MOVE_COMPILER_DEBUG";

pub const DEBUG_FLAG: &str = "debug";

pub const MOVE_COMPILER_WARN_OF_DEPRECATION_USE: &str = "MOVE_COMPILER_WARN_OF_DEPRECATION_USE";
pub const MOVE_COMPILER_WARN_OF_DEPRECATION_USE_FLAG: &str = "Wdeprecation";

pub const WARN_OF_DEPRECATION_USE_IN_APTOS_LIBS: &str = "WARN_OF_DEPRECATION_USE_IN_APTOS_LIBS";
pub const WARN_OF_DEPRECATION_USE_IN_APTOS_LIBS_FLAG: &str = "Wdeprecation-aptos";
