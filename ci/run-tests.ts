// Builds the project tests as a headless binary for the current platfrom
import { $ } from "bun";
import { createDir, globRename, validTarget } from "./util";
import { getData } from ".";
import type { Target } from ".";
import { resolve, join } from "path";

async function main() {
  const target = Bun.argv[2] as Target;
  if (!validTarget(target)) return;
  const { buildDir, reportsDir, tests } = getData(target);
  await createDir(reportsDir);
  const binPath = resolve(buildDir, tests.binary);
  console.log(`Running: ${binPath}" --headless ++ --test=determinism`);
  await $`"${binPath}" --headless ++ --test=determinism`.cwd(buildDir);

  console.log(`\nRenaming reports`);
  await globRename(
    join(reportsDir, "*report.txt"),
    join(reportsDir, `${target}-${Date.now()}-report.txt`)
  );
}

await main();
