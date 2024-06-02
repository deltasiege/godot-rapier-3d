// Runs the tests for a target and renames the report file to include the target name and the current date.
import { createDir, extract, validTarget } from "./util";
import { getData } from ".";
import type { Target } from ".";
import { resolve, join } from "path";

async function main() {
  const target = Bun.argv[2] as Target;
  if (!validTarget(target)) return;
  const data = getData(target);
  const { buildDir, reportsDir, tests, osName } = data;
  await createDir(reportsDir);

  const binPath =
    osName === "macos"
      ? await extractMacOSBinary(data)
      : resolve(buildDir, tests.binary);
  const args = [
    binPath,
    "--headless",
    "--no-window",
    "++",
    "--test=determinism",
    `--target=${target}`,
  ];
  console.log("Running:", args.join(" ") + "\n");
  const proc = Bun.spawn(args, { stdout: "inherit" });
  const text = await new Response(proc.stdout).text();
  console.log(text);
  await proc.exited;
}

async function extractMacOSBinary({ buildDir, tests }: any): Promise<string> {
  await extract(join(buildDir, tests.binary), buildDir);
  return join(
    buildDir,
    "godot-rapier-3d.app",
    "Contents",
    "MacOS",
    "godot-rapier-3d"
  );
}

await main();
