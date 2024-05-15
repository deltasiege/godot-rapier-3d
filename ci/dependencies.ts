// Downloads Godot and builds the project as a headless binary for the current platfrom
import {
  validTarget,
  downloadIfNotExists,
  downloadAndExtract,
  extract,
  globMove,
  createDir,
} from "./util";
import { getData } from ".";
import type { Target } from ".";
import { join } from "path";

async function main() {
  const target = Bun.argv[2] as Target;
  if (!validTarget(target)) return;
  const { ciBinDir, godot } = getData(target);

  // Godot
  await downloadAndExtract(godot.url, ciBinDir);

  // Export templates
  const etFileName = await downloadIfNotExists(
    godot.exportTemplatesUrl,
    ciBinDir
  ); // Need to DL to ciBinDir for caching
  if (!etFileName) return;
  await createDir(godot.exportTemplatesDir);
  await extract(join(ciBinDir, etFileName), godot.exportTemplatesDir);
  await globMove(
    join(godot.exportTemplatesDir, "templates", "**"),
    godot.exportTemplatesDir
  );
}

await main();
