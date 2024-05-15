// Builds the project tests as a headless binary for the current platfrom
import { $ } from "bun";
import { createDir, validTarget } from "./util";
import { getData } from ".";
import type { Target } from ".";
import { resolve } from "path";

async function main() {
  const timeout = setTimeout(() => {
    console.error("Bun timeout after 10 minutes");
    process.exit();
  }, 10 * 60 * 1000);

  const target = Bun.argv[2] as Target;
  if (!validTarget(target)) return;
  const { projectDir, projectBinExt, buildDir, ciBinDir, godot } =
    getData(target);
  const binPath = resolve(ciBinDir, godot.binary);
  const destPath = resolve(buildDir, `gr3d-tests.${projectBinExt}`);

  await openProject(binPath, projectDir);

  console.log("Sleeping for 2 seconds");
  await Bun.sleep(2000);

  await openProject(binPath, projectDir);

  await createDir(buildDir);
  await buildProject(target, binPath, projectDir, destPath);
  clearTimeout(timeout);
}

async function openProject(binPath: string, srcDir: string) {
  console.log("\nOpening project in Godot editor\n");
  console.log(
    `\n"${binPath}" --quit-after 20 --verbose --headless -e --path ${srcDir}\n`
  );
  await $`"${binPath}" --quit-after 20 --verbose --headless -e --path ${srcDir}`.nothrow();
}

async function buildProject(
  target: Target,
  binPath: string,
  srcDir: string,
  destPath: string
) {
  console.log(`\nBuilding tests for ${target}: \n`);

  console.log(
    `\n"${binPath}" --headless --path ${srcDir} --export-release tests--${target} ${destPath}`
  );
  await $`"${binPath}" --headless --path ${srcDir} --export-release tests--${target} ${destPath}`;
}

await main();
