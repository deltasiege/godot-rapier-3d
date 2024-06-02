// Downloads Godot and builds the project as a headless binary for the current platfrom
import {
  validTargets,
  extract,
  globMove,
  createDir,
  getFileNameFromUrl,
} from "./util";
import { getData } from ".";
import type { Target } from ".";
import { join } from "path";

async function main() {
  const targets = Bun.argv.slice(2) as Target[];
  console.log("Targets:", targets);
  if (!validTargets(targets)) return;
  const { ciBinDir, godot } = getData(targets[0]); // Assuming all targets are of the same OS, only need to fetch dependencies for one

  // Extract Godot
  const godotFileName = getFileNameFromUrl(godot.url);
  if (!godotFileName) return;
  extract(join(ciBinDir, godotFileName), ciBinDir);

  // Extract export templates
  const etFileName = getFileNameFromUrl(godot.exportTemplatesUrl);
  if (!etFileName) return;
  await createDir(godot.exportTemplatesDir);
  await extract(join(ciBinDir, etFileName), godot.exportTemplatesDir);
  await globMove(
    join(godot.exportTemplatesDir, "templates", "**"),
    godot.exportTemplatesDir
  );
}

await main();
