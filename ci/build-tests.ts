// Builds the project tests as a headless binary for the current platfrom
import { $ } from "bun";
import { createDir, validTarget } from "./util";
import { getData } from ".";
import type { Target } from ".";
import { resolve } from "path";

async function main() {
  const target = Bun.argv[2] as Target;
  if (!validTarget(target)) process.exit();
  const { projectDir, projectBinExt, buildDir, ciBinDir, godot } =
    getData(target);
  const binPath = resolve(ciBinDir, godot.binary);
  const destPath = resolve(buildDir, `gr3d-tests.${projectBinExt}`);

  await openProject(binPath, projectDir);

  console.log("---> Sleeping for 1 second");
  await Bun.sleep(1000);

  await buildProject(target, binPath, projectDir, destPath);
}

async function openProject(binPath: string, srcDir: string) {
  console.log("\n---> Opening project in Godot editor\n");
  const args = [
    binPath,
    "--quit-after",
    "2",
    "--headless",
    "-e",
    "--path",
    srcDir,
  ];
  console.log(args.join(" ") + "\n");
  const proc = Bun.spawn(args);
  await proc.exited;
}

async function buildProject(
  target: Target,
  binPath: string,
  srcDir: string,
  destPath: string
) {
  console.log(`\n---> Building tests for ${target}\n`);
  const args = [
    binPath,
    "--headless",
    "--path",
    srcDir,
    "--export-release",
    `tests--${target}`,
    destPath,
  ];
  console.log(args.join(" ") + "\n");
  const proc = Bun.spawn(args);
  await proc.exited;
}

main();
