// Downloads Godot and builds the project as a headless binary for the current platfrom
import { validTargets, downloadIfNotExists, downloadToDir } from "./util";
import { getData } from ".";
import type { Target } from ".";

async function main() {
  const targets = Bun.argv.slice(2) as Target[];
  console.log("Targets:", targets);
  if (!validTargets(targets)) return;
  const { ciBinDir, godot } = getData(targets[0]); // Assuming all targets are of the same OS, only need to fetch dependencies for one
  await downloadToDir(godot.url, ciBinDir); // Download Godot
  await downloadIfNotExists(godot.exportTemplatesUrl, ciBinDir); // Download export templates
}

await main();
