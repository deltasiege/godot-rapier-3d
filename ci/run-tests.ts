// Builds the project tests as a headless binary for the current platfrom
import { $ } from "bun";
import { createDir, validTarget } from "./util";
import { getData } from ".";
import type { Target } from ".";
import { resolve } from "path";

async function main() {
  const target = Bun.argv[2] as Target;
  if (!validTarget(target)) return;
  const { buildDir, reportsDir, tests } = getData(target);
  await createDir(reportsDir);
  const binPath = resolve(buildDir, tests.binary);
  await $`"${binPath}" --headless ++ --test=determinism`;
}

await main();
