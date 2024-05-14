// Downloads Godot and builds the project as a headless binary for the current platfrom
import {
  validTarget,
  downloadIfNotExists,
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
  const fileName = await downloadIfNotExists(
    godot.exportTemplatesUrl,
    ciBinDir
  );
  if (!fileName) return;
  await createDir(godot.exportTemplatesDir);
  await extract(join(ciBinDir, fileName), godot.exportTemplatesDir);
  await globMove(
    join(godot.exportTemplatesDir, "templates", "**"),
    godot.exportTemplatesDir
  );
}

await main();
