const path = require("node:path");

const repoRoot = path.resolve(__dirname, "../../..");

module.exports = {
  outputDir: path.join(
    repoRoot,
    "crates/z00z_storage/outputs/checkpoint/phase-110/playwright",
  ),
  use: process.env.Z00Z_PLAYWRIGHT_EXECUTABLE_PATH
    ? {
        launchOptions: {
          executablePath: process.env.Z00Z_PLAYWRIGHT_EXECUTABLE_PATH,
        },
      }
    : {},
};
