const path = require("node:path");

const repoRoot = path.resolve(__dirname, "../../..");

module.exports = {
  outputDir: path.join(
    repoRoot,
    "crates/z00z_storage/outputs/checkpoint/phase-110/playwright",
  ),
};
