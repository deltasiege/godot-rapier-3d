// Builds the project tests as a headless binary for the current platfrom
import { validTargets } from "./util";
import { getData } from ".";
import type { Target } from ".";
import { resolve } from "path";
import fsExtra from "fs-extra";

async function main() {
  const targets = Bun.argv.slice(2) as Target[];
  console.log("Targets:", targets);
  if (!validTargets(targets)) process.exit();
  const { projectDir } = getData(targets[0]);
  await deleteUnwantedFiles(projectDir);
  await buildAllTargets(targets);
}

async function deleteUnwantedFiles(projectDir: string) {
  const paths = ["addons/godot-rapier-3d/icons", "demos"];
  for (const path of paths) {
    const fullPath = resolve(projectDir, path);
    await fsExtra.remove(fullPath);
  }
}

async function buildAllTargets(targets: Target[]) {
  const promises = targets.map((target) => {
    const { projectDir, buildDir, ciBinDir, godot, projectBinExt } =
      getData(target);
    const binPath = resolve(ciBinDir, godot.binary);
    const destPath = resolve(buildDir, `gr3d-tests-${target}.${projectBinExt}`);
    return buildProject(target, binPath, projectDir, destPath);
  });
  await Promise.all(promises);
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
    "--no-window",
    "--path",
    srcDir,
    "--export-release",
    `tests--${target}`,
    destPath,
  ];
  console.log(args.join(" ") + "\n");
  const proc = Bun.spawn(args, { stdout: "inherit" });
  const text = await new Response(proc.stdout).text();
  console.log(text);
  await proc.exited;
}

main();
